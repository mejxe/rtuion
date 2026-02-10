# rtuion - minimalist study companion
RTuion [ˈtujɔn] is a simple terminal interface timer with some extra features.
## Features
- Simple and clean interface
- Works in terminal
- Pomodoro timer
- Flowmodoro timer
- Simple Progress Tracking - log your study sessions
- Extended Progress Tracking - sync your study sessions with [pixe.la](https://pixe.la/)

## Installation
### Build manually
```
cargo install --git https://github.com/mejxe/rtuion.git
```

### Release
This project uses cargo-dist for packing releases.
Simply check out the newest release and follow the steps there.
Alternatively if you have cargo installed you can download it with cargo binstall
```
cargo binstall --git https://github.com/mejxe/rtuion.git
```

## Stats Tracking/Pixela Integration
RTuion makes it possible to log your study sessions. To use it to the full potential you will have to make a [pixe.la](https://pixe.la/) account.
The service is free (altough I encourage you to support pixe.la creator) and works great for tracking your progress.

If you do not wish to use remote stats tracking, you can use the simple mode that will log your study sessions locally (you still have to provide username in the settings!).

## Gallery
![Timer](images/timer.png)
![Settings](images/settings.png)
![Stats](images/stats.png)


## License
rtuion is licensed under MIT license.
