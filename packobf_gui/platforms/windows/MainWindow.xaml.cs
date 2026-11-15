using System.Collections.ObjectModel;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.Windows.Storage.Pickers;
using Windows.ApplicationModel.DataTransfer;
using Windows.Graphics;
using Windows.UI.ViewManagement;

namespace PackObf.Windows;

public sealed partial class MainWindow : Window
{
    private static readonly VersionChoice[] Versions =
    [
        new("Any", 0),
        new("1.21.1", 34),
        new("1.21.2", 42),
        new("1.21.4", 46),
        new("1.21.5", 55),
        new("1.21.6", 63),
        new("1.21.7", 64),
        new("1.21.9", 69),
        new("1.21.11", 75),
        new("26.1", 84),
        new("26.2", 88),
        new("26.3", 97),
    ];

    private readonly Optimizer optimizer = new();
    private readonly DispatcherQueueTimer timer;
    private readonly Style accentButtonStyle;
    private readonly List<Optimizer.LogEntry> logs = [];
    private readonly ObservableCollection<LogRow> visibleLogs = [];
    private readonly CheckBox[] filters;
    private SolidColorBrush[] logBrushes = [];
    private bool followLogs = true;
    private bool interactionOpen;
    private bool closed;
    private string? pendingSummary;

    public MainWindow()
    {
        InitializeComponent();
        accentButtonStyle = OptimizeButton.Style;
        timer = DispatcherQueue.CreateTimer();
        timer.Interval = TimeSpan.FromMilliseconds(50);
        timer.Tick += (_, _) => Refresh();

        var defaults = Optimizer.Defaults;
        Compression.SelectedIndex = defaults.Compression;
        Shader.SelectedIndex = defaults.ShaderCompression;
        Sound.SelectedIndex = defaults.SoundQuality;
        foreach (var version in Versions)
        {
            var item = new ComboBoxItem { Content = version.Label, Tag = version.PackFormat };
            Version.Items.Add(item);
            if (version.PackFormat == defaults.TargetVersion)
                Version.SelectedItem = item;
        }
        Rename.IsOn = defaults.RenameFiles != 0;
        BlockUnzipping.IsOn = defaults.BlockUnzipping != 0;
        CorruptPng.IsOn = defaults.CorruptPngFiles != 0;
        Threads.Value = defaults.WorkerThreads;

        InputPath.TextChanged += (_, _) => InvalidateResult();
        CachePath.TextChanged += (_, _) => InvalidateResult();
        foreach (var choice in new[] { Compression, Shader, Sound, Version })
            choice.SelectionChanged += (_, _) => InvalidateResult();
        foreach (var toggle in new[] { Rename, BlockUnzipping, CorruptPng })
            toggle.Toggled += (_, _) => InvalidateResult();
        Threads.ValueChanged += (_, _) => InvalidateResult();

        filters = [InfoFilter, WarningFilter, ErrorFilter];
        foreach (var filter in filters)
        {
            filter.Checked += (_, _) => RefreshLogFilter();
            filter.Unchecked += (_, _) => RefreshLogFilter();
        }
        LogList.ItemsSource = visibleLogs;
        Root.ActualThemeChanged += (_, _) => UpdateLogBrushes();
        UpdateLogBrushes();
        Root.Loaded += SetInitialSize;

        Closed += (_, _) =>
        {
            closed = true;
            timer.Stop();
            optimizer.Dispose();
        };
    }

    private void SetInitialSize(object sender, RoutedEventArgs args)
    {
        Root.Loaded -= SetInitialSize;
        double scale = Root.XamlRoot.RasterizationScale;
        RectInt32 workArea = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Nearest).WorkArea;
        AppWindow.Resize(new SizeInt32(Math.Min((int)(720 * scale), workArea.Width),
            Math.Min((int)(800 * scale), workArea.Height)));
    }

    private async void Optimize(object sender, RoutedEventArgs args)
    {
        if (InputPath.Text.Length == 0 || timer.IsRunning)
            return;

        var settings = new Optimizer.Settings
        {
            Compression = (byte)Compression.SelectedIndex,
            ShaderCompression = (byte)Shader.SelectedIndex,
            SoundQuality = (byte)Sound.SelectedIndex,
            TargetVersion = (byte)((ComboBoxItem)Version.SelectedItem).Tag,
            RenameFiles = Convert.ToByte(Rename.IsOn),
            BlockUnzipping = Convert.ToByte(BlockUnzipping.IsOn),
            CorruptPngFiles = Convert.ToByte(CorruptPng.IsOn),
            WorkerThreads = double.IsNaN(Threads.Value) ? 0 : (uint)Math.Round(Threads.Value),
        };

        string? error = optimizer.Start(InputPath.Text, CachePath.Text.Length == 0 ? null : CachePath.Text, settings);
        if (error is not null)
        {
            await RunInteraction("Couldn’t start optimization", () => ShowMessage("Couldn’t start optimization", error));
            return;
        }

        pendingSummary = null;
        logs.Clear();
        visibleLogs.Clear();
        followLogs = true;
        timer.Start();
        Refresh();
    }

    private void InvalidateResult()
    {
        if (closed)
            return;

        pendingSummary = null;
        optimizer.Invalidate();
        Refresh();
    }

    private void Refresh()
    {
        var state = optimizer.Poll();
        if (Status.Text != state.Status)
        {
            Status.Text = state.Status;
            ToolTipService.SetToolTip(Status, state.Status);
        }
        Spinner.IsActive = state.Running;
        Spinner.Visibility = state.Running ? Visibility.Visible : Visibility.Collapsed;
        OptimizeButton.Visibility = state.Running ? Visibility.Collapsed : Visibility.Visible;
        OptimizeButton.IsEnabled = !state.Running && InputPath.Text.Length != 0;
        OptimizeButton.Style = state.CanSave ? null : accentButtonStyle;
        SaveButton.Visibility = state.CanSave ? Visibility.Visible : Visibility.Collapsed;

        if ((nuint)logs.Count < state.LogCount)
        {
            if (LogPanel.Visibility == Visibility.Visible)
                followLogs = AtEndOfLogs();

            for (nuint index = (nuint)logs.Count; index < state.LogCount; index++)
            {
                var entry = optimizer.Log(index);
                logs.Add(entry);
                AddVisibleLog(entry);
            }
            ScrollToLatestLog();
        }

        if (!state.Running)
            timer.Stop();

        if (state.Summary is not null)
        {
            pendingSummary = state.Summary;
            if (!interactionOpen)
                _ = ShowPendingSummary();
        }
    }

    private void ToggleLogs(object sender, RoutedEventArgs args)
    {
        bool shown = LogToggle.IsChecked == true;
        if (!shown)
            followLogs = AtEndOfLogs();
        LogToggle.Content = shown ? "Hide logs" : "Show logs";
        LogPanel.Visibility = shown ? Visibility.Visible : Visibility.Collapsed;
        ScrollToLatestLog();
    }

    private void RefreshLogFilter()
    {
        visibleLogs.Clear();
        foreach (var entry in logs)
            AddVisibleLog(entry);
        followLogs = true;
        ScrollToLatestLog();
    }

    private void AddVisibleLog(Optimizer.LogEntry entry)
    {
        int level = (int)entry.Level;
        if (filters[level].IsChecked == true)
            visibleLogs.Add(new LogRow(entry.Line, logBrushes[level]));
    }

    private bool AtEndOfLogs() =>
        FindScrollViewer(LogList) is not { } scroll || scroll.ScrollableHeight - scroll.VerticalOffset <= 2;

    private void ScrollToLatestLog()
    {
        if (followLogs && LogPanel.Visibility == Visibility.Visible && visibleLogs.Count > 0)
            LogList.ScrollIntoView(visibleLogs[^1]);
    }

    private static ScrollViewer? FindScrollViewer(DependencyObject element)
    {
        if (element is ScrollViewer scroll)
            return scroll;
        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(element); i++)
            if (FindScrollViewer(VisualTreeHelper.GetChild(element, i)) is { } child)
                return child;
        return null;
    }

    private void UpdateLogBrushes()
    {
        if (new AccessibilitySettings().HighContrast)
        {
            var foreground = new SolidColorBrush(new UISettings().GetColorValue(UIColorType.Foreground));
            logBrushes = [foreground, foreground, foreground];
        }
        else if (Root.ActualTheme == ElementTheme.Dark)
        {
            logBrushes = [new(Colors.LightSkyBlue), new(Colors.Gold), new(Colors.LightSalmon)];
        }
        else
        {
            logBrushes = [new(Colors.RoyalBlue), new(Colors.SaddleBrown), new(Colors.Firebrick)];
        }
        RefreshLogFilter();
    }

    private async void CopyLogs(object sender, RoutedEventArgs args) =>
        await RunInteraction("Couldn’t copy logs", () =>
        {
            var data = new DataPackage();
            data.SetText(optimizer.CopyableLogs());
            Clipboard.SetContent(data);
            Clipboard.Flush();
            return Task.CompletedTask;
        });

    private async void ChooseInput(object sender, RoutedEventArgs args) =>
        await PickInput("Select Resource Pack", ".zip", InputPath);

    private async void ImportCache(object sender, RoutedEventArgs args) =>
        await PickInput("Import Cache File", ".bin", CachePath);

    private Task PickInput(string title, string extension, TextBox destination) =>
        RunInteraction(title, async () =>
        {
            var picker = new FileOpenPicker(AppWindow.Id);
            picker.FileTypeFilter.Add(extension);
            var result = await picker.PickSingleFileAsync();
            if (!closed && result is not null)
                destination.Text = result.Path;
        });

    private async void CreateCache(object sender, RoutedEventArgs args) =>
        await RunInteraction("Create Cache File", async () =>
        {
            string? path = await PickSavePath("BIN files", ".bin", "cache");
            if (!closed && path is not null)
                CachePath.Text = path;
        });

    private async void SaveOutput(object sender, RoutedEventArgs args) =>
        await RunInteraction("Couldn’t save resource pack", async () =>
        {
            string? path = await PickSavePath("ZIP files", ".zip", "pack");
            if (!closed && path is not null && optimizer.Save(path) is { } error)
                await ShowMessage("Couldn’t save resource pack", error);
        });

    private async Task<string?> PickSavePath(string fileType, string extension, string name)
    {
        var picker = new FileSavePicker(AppWindow.Id)
        {
            SuggestedFileName = name,
            DefaultFileExtension = extension,
        };
        picker.FileTypeChoices.Add(fileType, [extension]);
        return (await picker.PickSaveFileAsync())?.Path;
    }

    private async Task RunInteraction(string errorTitle, Func<Task> action)
    {
        if (closed || interactionOpen)
            return;

        interactionOpen = true;
        try
        {
            await action();
        }
        catch (Exception error)
        {
            if (!closed)
                await ShowMessage(errorTitle, error.Message);
        }
        finally
        {
            interactionOpen = false;
        }
        await ShowPendingSummary();
    }

    private async Task ShowPendingSummary()
    {
        if (closed || interactionOpen || pendingSummary is not { } summary)
            return;

        pendingSummary = null;
        await RunInteraction("Optimization Complete", () => ShowMessage("Optimization Complete", summary));
    }

    private async Task ShowMessage(string title, string message)
    {
        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot,
            Title = title,
            Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap },
            CloseButtonText = "OK",
            DefaultButton = ContentDialogButton.Close,
        };
        await dialog.ShowAsync();
    }

    private sealed record VersionChoice(string Label, byte PackFormat);
}

public sealed record LogRow(string Line, Brush Foreground);
