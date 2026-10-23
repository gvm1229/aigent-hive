// Read only fixed public PE image regions; never inspect stack, heap, or model state.
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using Microsoft.Win32.SafeHandles;

public sealed class HiveParentImage : IDisposable
{
    private const uint QueryReadSynchronize = 0x00100410;
    private const uint StillActive = 259;
    private const ulong MaximumImage = 128UL * 1024 * 1024;
    private readonly SafeProcessHandle handle;
    private readonly ulong imageBase;
    public long CreationFileTime { get; private set; }
    public string ExecutablePath { get; private set; }

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern SafeProcessHandle OpenProcess(uint access, bool inherit, int processId);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetExitCodeProcess(SafeProcessHandle process, out uint code);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetProcessTimes(SafeProcessHandle process, out long created, out long exited, out long kernel, out long user);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool QueryFullProcessImageName(SafeProcessHandle process, uint flags, StringBuilder name, ref uint size);
    [DllImport("psapi.dll", SetLastError = true)]
    private static extern bool EnumProcessModulesEx(SafeProcessHandle process, [Out] IntPtr[] modules, uint bytes, out uint required, uint filter);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool ReadProcessMemory(SafeProcessHandle process, IntPtr address, [Out] byte[] bytes, UIntPtr size, out UIntPtr count);

    public HiveParentImage(int processId)
    {
        if (IntPtr.Size != 8 || processId <= 0) throw new ArgumentException("unsupported parent identity");
        handle = OpenProcess(QueryReadSynchronize, false, processId);
        if (handle.IsInvalid) throw new Win32Exception("parent process unavailable");
        try
        {
            long created, exited, kernel, user;
            if (!GetProcessTimes(handle, out created, out exited, out kernel, out user)) throw new Win32Exception("parent creation time unavailable");
            CreationFileTime = created;
            var name = new StringBuilder(32768);
            uint size = (uint)name.Capacity;
            if (!QueryFullProcessImageName(handle, 0, name, ref size) || size == 0 || size >= name.Capacity) throw new Win32Exception("parent image name unavailable");
            ExecutablePath = name.ToString();
            var modules = new IntPtr[1024];
            uint required;
            if (!EnumProcessModulesEx(handle, modules, (uint)(modules.Length * IntPtr.Size), out required, 3)
                || required == 0 || required > modules.Length * IntPtr.Size || modules[0] == IntPtr.Zero)
                throw new Win32Exception("parent image mapping unavailable");
            imageBase = unchecked((ulong)modules[0].ToInt64());
            RequireAlive();
        }
        catch
        {
            handle.Dispose();
            throw;
        }
    }

    public void RequireAlive()
    {
        uint code;
        if (!GetExitCodeProcess(handle, out code) || code != StillActive) throw new InvalidOperationException("parent process ended");
    }

    private byte[] Read(uint rva, int size)
    {
        if (size <= 0 || size > 64 * 1024 * 1024 || (ulong)rva + (ulong)size > MaximumImage)
            throw new ArgumentException("parent image read outside fixed bounds");
        RequireAlive();
        ulong address = checked(imageBase + rva);
        var bytes = new byte[size];
        UIntPtr count;
        if (!ReadProcessMemory(handle, new IntPtr(unchecked((long)address)), bytes, new UIntPtr((uint)size), out count)
            || count.ToUInt64() != (ulong)size)
            throw new Win32Exception("parent image region unavailable");
        RequireAlive();
        return bytes;
    }

    private static bool Matches(byte[] bytes, string expected)
    {
        if (expected == null || expected.Length != 64) throw new ArgumentException("invalid public image digest");
        using (var sha = SHA256.Create())
        {
            string actual = BitConverter.ToString(sha.ComputeHash(bytes)).Replace("-", "").ToLowerInvariant();
            return String.Equals(actual, expected, StringComparison.Ordinal);
        }
    }

    public bool HeaderMatches(int size, ulong preferredBase, int baseOffset, string expected)
    {
        if (size > 16384 || baseOffset < 0 || (long)baseOffset + 8 > size) throw new ArgumentException("invalid public header bound");
        var bytes = Read(0, size);
        ulong declaredBase = BitConverter.ToUInt64(bytes, baseOffset);
        if (declaredBase != preferredBase && declaredBase != imageBase) return false;
        Array.Copy(BitConverter.GetBytes(preferredBase), 0, bytes, baseOffset, 8);
        return Matches(bytes, expected);
    }

    public static uint[] DecodeRelocations(string encoded, int expectedCount)
    {
        if (encoded == null || encoded.Length > 600000 || expectedCount < 0 || expectedCount > 100000)
            throw new ArgumentException("invalid relocation bounds");
        var bytes = Convert.FromBase64String(encoded);
        var locations = new List<uint>();
        ulong previous = 0, delta = 0;
        int shift = 0;
        foreach (byte value in bytes)
        {
            if (shift >= 35) throw new ArgumentException("relocation overflow");
            delta |= (ulong)(value & 127) << shift;
            if ((value & 128) != 0) { shift += 7; continue; }
            if (delta == 0 || previous + delta + 8 > MaximumImage || locations.Count >= expectedCount)
                throw new ArgumentException("invalid relocation location");
            previous += delta;
            locations.Add((uint)previous);
            shift = 0;
            delta = 0;
        }
        if (shift != 0 || locations.Count != expectedCount) throw new ArgumentException("incomplete relocations");
        return locations.ToArray();
    }

    public bool SectionMatches(ulong preferredBase, uint rva, int size, string expected,
        uint[] relocations, int importOffset, int importSize)
    {
        if (rva < 1024 || relocations == null || relocations.Length > 100000
            || importOffset < 0 || importSize < 0 || (long)importOffset + importSize > size)
            throw new ArgumentException("invalid public section bounds");
        var bytes = Read(rva, size);
        ulong delta = unchecked(imageBase - preferredBase);
        uint previous = 0;
        foreach (uint location in relocations)
        {
            if (location <= previous) throw new ArgumentException("unordered public relocations");
            previous = location;
            if (location < rva || (ulong)location >= (ulong)rva + (ulong)size) continue;
            if ((ulong)location + 8 > (ulong)rva + (ulong)size) throw new ArgumentException("relocation escaped public section");
            int offset = (int)(location - rva);
            ulong original = unchecked(BitConverter.ToUInt64(bytes, offset) - delta);
            Array.Copy(BitConverter.GetBytes(original), 0, bytes, offset, 8);
        }
        Array.Clear(bytes, importOffset, importSize);
        bool result = Matches(bytes, expected);
        RequireAlive();
        return result;
    }

    public void Dispose() { handle.Dispose(); }
}
