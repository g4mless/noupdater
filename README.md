# NoUpdater
Easy way to disable (and also restore*) windows update within just 3 steps

*Restore function only works when you also disable windows update using this software ;)

<video src="assets/demo.mp4" controls width="100%"></video>

this project is partially vibe-coded, btw. i leave gpt & claude to create the ui and i do the logic for disabling the update. it comes from my experience tinkering with how to shut up windows update

## What my software does?
- Disable services that related to windows update (wuauserv, UsoSvc, WaaSMedicSvc, BITS, DoSvc)
- Corrupt a.k.a changing wuauserv ImagePath to make the windows service can't start the service.
- Add some Policies like NoAutoUpdate, WSUS redirection, block WU internet
- Lastly, Disable Update Orchestrator and WaaS Medic tasks