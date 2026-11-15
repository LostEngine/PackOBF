using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace PackObf.Windows;

internal sealed class Optimizer : IDisposable
{
    private const string Library = "packobf_gui";
    private readonly AppHandle handle = Native.Create();

    internal static Settings Defaults => Native.Defaults();

    internal string? Start(string input, string? cache, Settings settings) =>
        ReadString(Native.Start(handle, input, cache, settings));

    internal void Invalidate() => Native.Invalidate(handle);

    internal Snapshot Poll()
    {
        NativeSnapshot state = Native.Poll(handle);
        return new Snapshot(ReadString(state.Status)!, ReadString(state.Summary),
            state.LogCount, state.Running != 0, state.CanSave != 0);
    }

    internal LogEntry Log(nuint index)
    {
        NativeLog log = Native.Log(handle, index);
        return new LogEntry(ReadString(log.Line)!, log.Level);
    }

    internal string CopyableLogs() => ReadString(Native.CopyableLogs(handle))!;

    internal string? Save(string path) => ReadString(Native.Save(handle, path));

    public void Dispose() => handle.Dispose();

    private static string? ReadString(nint value)
    {
        try
        {
            return Marshal.PtrToStringUTF8(value);
        }
        finally
        {
            Native.FreeString(value);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct Settings
    {
        public byte Compression;
        public byte ShaderCompression;
        public byte SoundQuality;
        public byte TargetVersion;
        public byte RenameFiles;
        public byte BlockUnzipping;
        public byte CorruptPngFiles;
        public uint WorkerThreads;
    }

    internal readonly record struct Snapshot(string Status, string? Summary, nuint LogCount,
        bool Running, bool CanSave);

    internal enum Level : byte { Info, Warning, Error }

    internal readonly record struct LogEntry(string Line, Level Level);

    [StructLayout(LayoutKind.Sequential)]
    private struct NativeSnapshot
    {
        public nint Status;
        public nint Summary;
        public nuint LogCount;
        public byte Running;
        public byte CanSave;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct NativeLog
    {
        public nint Line;
        public Level Level;
    }

    private sealed class AppHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        public AppHandle() : base(ownsHandle: true) { }

        protected override bool ReleaseHandle()
        {
            Native.Destroy(handle);
            return true;
        }
    }

    private static class Native
    {
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_defaults")]
        internal static extern Settings Defaults();

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_create")]
        internal static extern AppHandle Create();

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_destroy")]
        internal static extern void Destroy(nint app);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_start")]
        internal static extern nint Start(AppHandle app, [MarshalAs(UnmanagedType.LPUTF8Str)] string input,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? cache, Settings settings);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_invalidate")]
        internal static extern void Invalidate(AppHandle app);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_poll")]
        internal static extern NativeSnapshot Poll(AppHandle app);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_log")]
        internal static extern NativeLog Log(AppHandle app, nuint index);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_copyable_logs")]
        internal static extern nint CopyableLogs(AppHandle app);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_save")]
        internal static extern nint Save(AppHandle app, [MarshalAs(UnmanagedType.LPUTF8Str)] string output);

        [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "packobf_gui_free_string")]
        internal static extern void FreeString(nint value);
    }
}
