#ifndef AppVersion
  #define AppVersion "1.0.0"
#endif

[Setup]
AppId={{6E7C2B1A-4D3F-4F8E-9B2A-5C1D0E7F3A42}
AppName=PO Studio
AppVersion={#AppVersion}
AppPublisher=PO Studio
DefaultDirName={autopf}\PO Studio
DefaultGroupName=PO Studio
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=PO-Studio-{#AppVersion}-windows-setup
SetupIconFile=..\..\assets\icon\icon.ico
UninstallDisplayIcon={app}\po-studio.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible arm64
ArchitecturesInstallIn64BitMode=x64compatible arm64
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\po-studio.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\PO Studio"; Filename: "{app}\po-studio.exe"
Name: "{group}\{cm:UninstallProgram,PO Studio}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\PO Studio"; Filename: "{app}\po-studio.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\po-studio.exe"; Description: "{cm:LaunchProgram,PO Studio}"; Flags: nowait postinstall skipifsilent
