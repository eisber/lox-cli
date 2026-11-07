param([Parameter(Mandatory=$true)][string]$ResultPath)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase
$form = New-Object Windows.Window
$form.Title = 'Lox UX owned fixture'
$form.Width = 540
$form.Height = 230
$form.WindowStartupLocation = 'CenterScreen'
$panel = New-Object Windows.Controls.StackPanel
$panel.Margin = '16'
$form.Content = $panel
$textbox = New-Object Windows.Controls.TextBox
[Windows.Automation.AutomationProperties]::SetName($textbox, 'Fixture value')
$textbox.Height = 28
$panel.Children.Add($textbox) | Out-Null
$commit = New-Object Windows.Controls.Button
$commit.Content = 'Record value'
$commit.Height = 30
$commit.Margin = '0,12,0,0'
$commit.Add_Click({
    [IO.File]::WriteAllText(
        $ResultPath,
        (@{value=$textbox.Text;pid=$PID;session=[Diagnostics.Process]::GetCurrentProcess().SessionId} | ConvertTo-Json),
        [Text.UTF8Encoding]::new($false)
    )
})
$panel.Children.Add($commit) | Out-Null
foreach ($number in 1, 2) {
    $duplicate = New-Object Windows.Controls.Button
    $duplicate.Content = 'Ambiguous'
    $duplicate.Height = 28
    $panel.Children.Add($duplicate) | Out-Null
}
[void]$form.ShowDialog()
