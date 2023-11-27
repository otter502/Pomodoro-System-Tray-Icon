# Pomodoro Timer

this is yet another version of a pomodoro timer but this time its on the windows system tray

this uses the powershell msg command to alert you about the time

this can only work on windows due to how the msg command is setup but it should hopefully be simple enough to fork this and modify it

in order to install it you need to use cargo to build the exe file (cargo build --release) which will put the exe in the newly created target/release/ folder
now you have an exe which acts as the pomodoro timer and you can choose to run it automatically.