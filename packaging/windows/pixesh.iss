; Inno Setup script for Pixesh. `pixesh.exe` is copied next to this file by
; scripts/package-windows.sh before compilation.
#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif

#define MyAppName "Pixesh"
#define MyAppPublisher "EnotCoder"
#define MyAppURL "https://github.com/EnotCoder/pixesh"
#define MyAppExeName "pixesh.exe"

[Setup]
AppId={{FA3E2AD1-8231-494E-8D30-4B5B72398DA8}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=pixesh-{#MyAppVersion}-setup
SetupIconFile=..\icons\pixesh.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add Pixesh to the system PATH (use `pixesh` in the terminal)"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associatepng"; Description: "Open PNG images with Pixesh"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "pixesh.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\icons\pixesh.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pixesh.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pixesh.ico"; Tasks: desktopicon

[Registry]
; System-wide association matches the elevated Program Files installation.
Root: HKLM; Subkey: "Software\Classes\.png\OpenWithProgids"; ValueType: string; ValueName: "Pixesh.Image"; ValueData: ""; Tasks: associatepng; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\Pixesh.Image"; ValueType: string; ValueName: ""; ValueData: "Pixesh image"; Tasks: associatepng; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Classes\Pixesh.Image\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\pixesh.ico"; Tasks: associatepng; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Classes\Pixesh.Image\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associatepng; Flags: uninsdeletekey

[Code]
const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure AddToPath;
var
  OldPath, AppPath, Needle: String;
  P: Integer;
begin
  AppPath := ExpandConstant('{app}');
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OldPath) then
    Exit;
  Needle := ';' + AppPath + ';';
  P := Pos(Needle, ';' + OldPath + ';');
  if P = 0 then
    RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OldPath + ';' + AppPath);
end;

procedure RemoveFromPath;
var
  OldPath, AppPath, NewPath: String;
  P: Integer;
begin
  AppPath := ExpandConstant('{app}');
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OldPath) then
    Exit;
  P := Pos(';' + AppPath + ';', ';' + OldPath + ';');
  if P = 0 then
    Exit;
  NewPath := Copy(OldPath, 1, P - 1) + Copy(OldPath, P + 1 + Length(AppPath), MaxInt);
  RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', NewPath);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    AddToPath;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveFromPath;
end;
