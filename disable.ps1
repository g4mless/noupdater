Stop-Service -Name wuauserv -Force -ErrorAction SilentlyContinue
Stop-Service -Name UsoSvc -Force -ErrorAction SilentlyContinue
Stop-Service -Name WaaSMedicSvc -Force -ErrorAction SilentlyContinue
Stop-Service -Name BITS -Force -ErrorAction SilentlyContinue
Stop-Service -Name DoSvc -Force -ErrorAction SilentlyContinue

Set-Service -Name wuauserv -StartupType Disabled
Set-Service -Name UsoSvc -StartupType Disabled
Set-Service -Name WaaSMedicSvc -StartupType Disabled
Set-Service -Name BITS -StartupType Disabled
Set-Service -Name DoSvc -StartupType Disabled

New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Force
New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Force

Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "NoAutoUpdate" -Value 1 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "AUOptions" -Value 1 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Name "WUServer" -Value "http://127.0.0.1"
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Name "WUStatusServer" -Value "http://127.0.0.1"
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Name "DoNotConnectToWindowsUpdateInternetLocations" -Value 1 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate" -Name "DisableDualScan" -Value 1 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" -Name "UseWUServer" -Value 1 -Type DWord

Disable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Scheduled Start" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\AUScheduledInstall" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Automatic App Update" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\Scheduled Start With Network" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\WindowsUpdate\UPR" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\Schedule Scan" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\Schedule Scan Static Task" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\USO_UxBroker_Display" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\UpdateOrchestrator\UpdateModelTask" -ErrorAction SilentlyContinue
Disable-ScheduledTask -TaskName "\Microsoft\Windows\WaaSMedic\PerformRemediation" -ErrorAction SilentlyContinue
