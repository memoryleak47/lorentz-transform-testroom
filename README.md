Lorentz Transform Testroom
==========================

This project is a simple simulator of the lorentz transform applied to a 2D world.
You can specify a scene (consisting of objects and their movement) in a toml file - and then observe them from different perspectives.
Have a look into the `examples` folder for inspiration.

Running it
----------

You'll need to have Rust (and cargo) installed to use this software.
You can run the simulator using `cargo run <config-file>`.

Try out `cargo run --release examples/twin-paradox.toml` for example!

Changing the Reference Frame
----------------------------

You can add `follow = "me"` to one object in the config-file to see the scene simulated from the viewpoint of that object.

Note that the space will change quite a bit when the object is accelerated!
So it's often more insightful to choose a non-accelerated observer.
