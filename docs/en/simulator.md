# Simulator – design

> **Status:** design and skeleton (`crates/sim`). The simulator is built
> once the Core repository has the virtual machine (VM) that runs compiled
> programs. This page describes the plan.

*Deutsche Version: [../de/simulator.md](../de/simulator.md)*

## Goals

1. **Test without a robot.** Students run their program with
   `ekips run --sim mission.ekips` and watch a 2D robot drive on the FLL mat.
2. **Same program, same behaviour.** A program that works in the simulator
   should behave the same on the hub (within the limits of a simple physics
   model).
3. **Runs everywhere:** Windows, Linux (x86_64 and aarch64 / Raspberry Pi),
   macOS, and **in the browser** via WebAssembly, so schools need no install.
4. **Reproducible.** The same program and start position always give the same
   result. This makes the simulator usable in tests and for comparing
   solutions.

Non-goals (for now): 3D, realistic collisions with mission models, wheel
slip, simulating the LEGO firmware.

## Architecture

```text
 .ekips source ──ekips-compiler──▶ bytecode ──▶ ekips-vm (Core)
                                                   │
                                     Hardware trait (ekips-hub-api)
                                   ┌───────────────┴───────────────┐
                                   ▼                               ▼
                       SimHardware (this crate)          firmware (on the hub)
                                   │
                                World
                     (mat, robot, devices, clock)
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
         native frontend (window,          browser frontend
         `ekips run --sim`)                (WASM + canvas)
```

- The **VM** never touches hardware. It calls the `Hardware` trait from
  `ekips-hub-api`. Commands that take time (`drive_straight`,
  `motor_run_angle`, …) only start a movement; the VM polls `*_done` and runs
  other tasks in between.
- **`SimHardware`** implements `Hardware` on top of a `World`.
- **`World`** holds the mat, the robot (pose, motors, sensors) and a
  simulated clock. It advances in fixed steps of `STEP_MS = 10` ms.
- **Frontends** only draw the world and feed input (start/stop, place the
  robot). They live in separate crates so `ekips-sim` keeps building for
  `wasm32-unknown-unknown` without platform dependencies (checked in CI).

### Main loop

```text
loop {
    vm.run_until_blocked(&mut sim_hardware);   // run program tasks
    world.step(STEP_MS);                       // move motors, robot, sensors
    frontend.draw(&world);                     // native or browser, optional
}
```

Without a frontend ("headless") the loop runs as fast as possible. This is
used for tests and could later check missions automatically.

## World model

### Coordinates

The same conventions as `ekips-hub-api` (implemented in `crates/sim`):

- millimetres, degrees, milliseconds;
- origin in the **bottom-left corner** of the mat, `x` to the right, `y` away
  from the student;
- heading `0°` points along `+y`; **positive = clockwise**, like `hub.heading`.

### Mat

- Default size: FLL mat, about **2362 mm × 1143 mm** (`Mat::FLL`).
- A background image (PNG/JPEG of the season's mat) is scaled onto it.
  Images are loaded by the frontend and passed in as RGB pixels, so the core
  crate stays free of image decoders.
- Outside the mat the table border counts as a wall for the distance sensor.

### Robot

- Configured by the program itself: `new DriveBase(left, right, 56mm, 112mm)`
  gives wheel diameter and axle track.
- Movement: differential drive (`Pose::advance`, already implemented): both
  wheels roll a distance per step, the robot follows the resulting arc.
- Motors: speed in % is mapped to degrees per second (SPIKE medium motor
  roughly 1000 °/s at 100 %), with a simple acceleration ramp. Movements end
  exactly on target to keep the results reproducible.
- Sensor positions relative to the robot centre are part of a **robot
  description** (default: color sensor in front, distance sensor in front,
  force sensor in front). Later students can edit it.

### Sensors

| Sensor | Simulation |
|---|---|
| Color sensor | Samples the mat image under the sensor (small area average) and maps the RGB value to the nearest `Color` and to a reflection in % |
| Distance sensor | Ray from the sensor in heading direction against the table border and optional obstacles (rectangles); `None` above 200 cm |
| Force sensor | `pressed` when the front of the robot touches the table border or an obstacle |
| Gyro (`hub.heading`) | The exact simulated heading (optionally with small noise later) |
| Battery | Constant 100 % |

### Hub output

`hub.show`, `hub.beep` and `console.log` are shown in the frontend
(light-matrix preview, sound or a note in the log, console panel).

## Frontends

| Frontend | Technology (proposal) | Notes |
|---|---|---|
| Native window | `minifb` or `pixels`/`winit` – small, pure Rust, works on Raspberry Pi | Started by `ekips run --sim`. Must also work without a GPU. |
| Browser | `wasm-bindgen` + HTML canvas | Hosted as a static web page; editor + simulator in one page. |
| Headless | none | `ekips run --sim --headless` prints a trace (time, pose, log) for tests. |

## Student-facing texts

All messages (e.g. "The robot left the mat", "Port C has no motor") exist in
English and German, like everywhere in the tools. Hardware errors come from
`HardwareError` and are translated in the CLI / frontend.

## Phases

1. **Now:** skeleton – mat, pose, drive kinematics, WASM build in CI.
2. **When the Core VM exists:** `World`, `SimHardware`, headless runner,
   `ekips run --sim --headless`.
3. Native window frontend.
4. Browser frontend (WASM) with mat images.
5. Robot description editor, obstacles, mission models.

## Open questions

- Exact motor model (speed curve, acceleration) – measure on a real hub.
- Should mat images be shipped with the tools (licensing of FLL mats)?
- How to describe obstacles/mission models (simple JSON file?).
