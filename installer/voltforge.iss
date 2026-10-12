; Instalador de VoltForge para Windows (Inno Setup 6).
;
; La integración continua lo compila así:
;   iscc /DAppVersion=0.2.0-alpha.1 /DAppVersionNum=0.2.0.1 installer\voltforge.iss
; El resultado queda en target\installer\.

#ifndef AppVersion
  #define AppVersion "0.0.0-dev"
#endif
#ifndef AppVersionNum
  #define AppVersionNum "0.0.0.0"
#endif

[Setup]
; El identificador no debe cambiar nunca: así Windows reconoce las
; actualizaciones como el mismo programa.
AppId={{6A0E3C52-4D7B-4E35-9B8C-2F1D5C7A9E41}
AppName=VoltForge
AppVersion={#AppVersion}
AppVerName=VoltForge {#AppVersion}
AppPublisher=Proyecto VoltForge
AppPublisherURL=https://github.com/JAAM08/voltforge
AppSupportURL=https://github.com/JAAM08/voltforge/issues
VersionInfoVersion={#AppVersionNum}
VersionInfoDescription=Instalador de VoltForge
DefaultDirName={autopf}\VoltForge
DefaultGroupName=VoltForge
DisableProgramGroupPage=yes
; Se instala para el usuario sin pedir permisos de administrador; quien los
; tenga puede instalarlo para todos.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\target\installer
OutputBaseFilename=VoltForge-{#AppVersion}-instalador
SetupIconFile=..\assets\icon\voltforge.ico
UninstallDisplayIcon={app}\VoltForge.exe
UninstallDisplayName=VoltForge
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "escritorio"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "asociar"; Description: "Abrir los proyectos .vfp con VoltForge (doble clic)"; GroupDescription: "Archivos:"

[Files]
Source: "..\target\release\voltforge-app.exe"; DestDir: "{app}"; DestName: "VoltForge.exe"; Flags: ignoreversion
; Línea de comandos, para scripts y agentes.
Source: "..\target\release\voltforge.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\VoltForge"; Filename: "{app}\VoltForge.exe"
Name: "{autodesktop}\VoltForge"; Filename: "{app}\VoltForge.exe"; Tasks: escritorio

[Registry]
Root: HKA; Subkey: "Software\Classes\.vfp"; ValueType: string; ValueName: ""; ValueData: "VoltForge.Proyecto"; Flags: uninsdeletevalue; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\.vfp\OpenWithProgids"; ValueType: string; ValueName: "VoltForge.Proyecto"; ValueData: ""; Flags: uninsdeletevalue; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\VoltForge.Proyecto"; ValueType: string; ValueName: ""; ValueData: "Proyecto de VoltForge"; Flags: uninsdeletekey; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\VoltForge.Proyecto\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\VoltForge.exe,0"; Tasks: asociar
Root: HKA; Subkey: "Software\Classes\VoltForge.Proyecto\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\VoltForge.exe"" ""%1"""; Tasks: asociar

[Run]
Filename: "{app}\VoltForge.exe"; Description: "{cm:LaunchProgram,VoltForge}"; Flags: nowait postinstall skipifsilent
