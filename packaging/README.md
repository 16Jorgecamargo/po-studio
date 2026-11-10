# Instaladores do PO Studio

Os instaladores prontos ficam na pasta `dist/`.

| Sistema | Arquivo | Como instalar |
|---|---|---|
| macOS (Apple Silicon e Intel) | `PO-Studio-<versão>-macos.dmg` | Abrir o .dmg e arrastar o PO Studio para Aplicativos. Na primeira vez o macOS bloqueia o app, porque ele não tem assinatura paga da Apple: abrir Ajustes do Sistema → Privacidade e Segurança e clicar em Abrir Mesmo Assim. |
| Windows | `PO-Studio-<versão>-windows-setup.exe` | Executar e seguir o assistente. Também há uma versão portátil `...-windows-portable.exe`. |
| Linux (Debian/Ubuntu) | `po-studio_<versão>_amd64.deb` | Dar dois cliques ou rodar `sudo apt install ./po-studio_<versão>_amd64.deb`. |
| Linux (qualquer distribuição) | `PO-Studio-<versão>-linux-x86_64.tar.gz` | Extrair e rodar `./install.sh` (instala para o usuário, sem precisar de senha). |

## Gerar de novo

- macOS: `bash packaging/macos/build_macos.sh` (para Intel e Apple Silicon juntos: `TARGETS="aarch64-apple-darwin x86_64-apple-darwin" bash packaging/macos/build_macos.sh`).
- Linux: `bash packaging/linux/build_linux.sh` num Linux, ou pelo Docker a partir do Mac.
- Windows: `pwsh -File packaging/windows/build_windows.ps1` num Windows com Inno Setup 6, ou `bash packaging/windows/build_windows_docker.sh` a partir do Mac com Docker.
- Os três de uma vez: enviar a pasta para o GitHub e rodar o fluxo "installers" (ou criar uma tag `v1.0.0`).

O ícone é gerado por `python3 packaging/make_icon.py assets/icon`.

O app no Mac é assinado só localmente; para distribuir sem aviso do macOS é preciso uma conta de desenvolvedor da Apple para assinar e notarizar.
