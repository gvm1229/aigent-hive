// Dedicated console handles leave the updater's JSON stdin/stdout pipes unchanged.
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Win32.SafeHandles;

public sealed class HiveUpdateConsole : IDisposable
{
    private SafeFileHandle input;
    private SafeFileHandle output;
    private bool attached;
    private bool ended;
    private Process parent;
    private Process caller;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool AttachConsole(uint processId);
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool FreeConsole();
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint GetConsoleProcessList([Out] uint[] processes, uint count);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern SafeFileHandle CreateFile(string name, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool ReadConsole(SafeFileHandle handle, [Out, MarshalAs(UnmanagedType.LPArray, ArraySubType = UnmanagedType.U2)] char[] text, uint capacity, out uint read, IntPtr control);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool WriteConsole(SafeFileHandle handle, string text, uint count, out uint written, IntPtr reserved);

    public HiveUpdateConsole(int verifiedParentId, int callerId, long parentCreated, long callerCreated)
    {
        if (verifiedParentId <= 0 || callerId <= 0) throw new ArgumentException("invalid console parent");
        // The caller verifies the exact parent before this call. Never search for another console.
        if (!AttachConsole((uint)verifiedParentId))
        {
            if (Marshal.GetLastWin32Error() != 5) throw new Win32Exception("parent console unavailable");
            // PowerShell can already own a console. Keep it only when the verified parent shares it.
            if (!ContainsParent((uint)verifiedParentId) && (!FreeConsole() || !AttachConsole((uint)verifiedParentId)))
                throw new Win32Exception("verified parent console unavailable");
        }
        attached = true;
        try
        {
            if (!ContainsParent((uint)verifiedParentId)) throw new InvalidOperationException("console parent differs");
            parent = Process.GetProcessById(verifiedParentId);
            caller = Process.GetProcessById(callerId);
            // Open handles now so a reused process id cannot revive a disconnected session.
            if (parent.Handle == IntPtr.Zero || caller.Handle == IntPtr.Zero
                || parent.StartTime.ToUniversalTime().ToFileTimeUtc() != parentCreated
                || Math.Abs(caller.StartTime.ToUniversalTime().ToFileTimeUtc() - callerCreated) > 10)
                throw new InvalidOperationException("console process identity changed");
            input = CreateFile("CONIN$", 0xC0000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero);
            output = CreateFile("CONOUT$", 0xC0000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero);
            if (input.IsInvalid || output.IsInvalid) throw new Win32Exception("dedicated console handles unavailable");
        }
        catch { Dispose(); throw; }
    }

    private static bool ContainsParent(uint processId)
    {
        var processes = new uint[4096];
        uint count = GetConsoleProcessList(processes, (uint)processes.Length);
        if (count == 0 || count > processes.Length) return false;
        for (int index = 0; index < count; index++) if (processes[index] == processId) return true;
        return false;
    }

    public void Write(string text)
    {
        if (ended || text == null || text.Length > 262144) throw new ArgumentException("invalid console output");
        uint written;
        if (!WriteConsole(output, text, (uint)text.Length, out written, IntPtr.Zero) || written != text.Length)
            throw new Win32Exception("parent console output ended");
    }

    public string ReadLine(int remainingMilliseconds)
    {
        if (ended) return null;
        if (remainingMilliseconds < 0 || remainingMilliseconds > 420000) throw new ArgumentException("invalid question deadline");
        if (remainingMilliseconds == 0) { ended = true; return null; }
        var elapsed = Stopwatch.StartNew();
        var read = Task.Run(() => ReadLineBlocking());
        while (!read.Wait(Math.Min(100, remainingMilliseconds)))
        {
            if (parent.HasExited || caller.HasExited || elapsed.ElapsedMilliseconds >= remainingMilliseconds)
            {
                ended = true;
                // The helper exits after replying EOF. Its background read cannot delay the CLI's journal.
                return null;
            }
        }
        if (parent.HasExited || caller.HasExited || elapsed.ElapsedMilliseconds >= remainingMilliseconds) { ended = true; return null; }
        return read.Result;
    }

    private string ReadLineBlocking()
    {
        var line = new StringBuilder();
        bool tooLong = false;
        while (true)
        {
            var chunk = new char[4096];
            uint read;
            if (!ReadConsole(input, chunk, 4096, out read, IntPtr.Zero)) throw new Win32Exception("parent console input ended");
            if (read == 0) return null;
            // ReadConsole returns a count, not a null-terminated string.
            if (read > chunk.Length) throw new InvalidOperationException("invalid console read length");
            string text = new string(chunk, 0, checked((int)read));
            if (text.IndexOf((char)26) >= 0 || text.IndexOf((char)3) >= 0) return null;
            if (!tooLong && line.Length + text.Length <= 65536) line.Append(text);
            else tooLong = true;
            if (text.IndexOf('\n') >= 0)
            {
                if (tooLong || Encoding.UTF8.GetByteCount(line.ToString()) > 65536)
                    return "\0invalid-overlong-answer\n";
                return line.ToString();
            }
        }
    }

    public void Dispose()
    {
        ended = true;
        if (attached) { FreeConsole(); attached = false; }
        if (input != null) input.Dispose();
        if (output != null) output.Dispose();
        if (parent != null) parent.Dispose();
        if (caller != null) caller.Dispose();
    }
}
