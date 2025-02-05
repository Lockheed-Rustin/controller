
# Simulation controller
The simulation controller is Lorenzo Ferranti's individual contribution. It's composed of:
- `SimulationControllerApp`: an eframe app that renders the UI you can use to interact with the simulation;
- `SimulationController`: Acts like a back-end, exposing APIs to interact with the simulation to the SimulationControllerApp (this struct is not present in this repo, you can find it in Lockheed-Rustin/drone-network);
- `SimulationData`: Shared data about the simulation, it cointains the logs and statistics associateed to each node and the SimulationController itself;
- Receiver threads: Threads spawned by SimulationControllerApp, their job is to receive DroneEvents, ClientEvents and ServerEvents sent by nodes and respond accordingly by updating the SimulationData.

## How to use it
Before starting the simulation, make sure your `config.toml` contains a valid topology configuration. After you `cargo run` the app, you should see a window. You can navigate the different sections of the app by clicking on the top menu. Here's what you'll find in each seection:
- `Control`: In this section you can open a dedicated window for each node in the simulation. The window lets you see the stats and logs about that node and lets you perform actions, like telling a client to send a message or telling a drone to crash.
- `Topology`: In this section you can see the topology of the simulated network. You can:
  - zoom in and out using `ctrl + mouse wheel`;
  - rearrange the placement of nodes (since the node-placing algorithm is not very good) by clicking and dragging them;
  - move the camera by clicking and dragging the background.
