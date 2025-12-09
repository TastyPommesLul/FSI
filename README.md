## FSI
FSI (Fabric Server Installer/Initializer), is a small CLI tool to download and install/initialize Minecraft Fabric Servers.

## Usage
Download the latest release from the releases tab.<br>
Then go into a terminal and run the executable.<br>
It will ask you a few questions and then download and install (and run) the server.

### Extending
Currently it only supports Linux and Windows, 1.21–1.21.10 only with the current latest Fabric version (0.18.3).<br>
To add more versions, add them into the `versions` array in `main.rs` file.<br>

###### if you want to add more OS's feel free to edit the code, compile for the specific OS and submit a PR.
