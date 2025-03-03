# Spell

Spell is a framework that provides the necessary tooling to create your own
shells for wayland compositors (like Hyprland). Except like other widget systems,
it doesn't simplify the process of creating gtk widgets for your setup, rather
it provides the thin abstraction layer or "glue" between wayland protocols and
slint declarative language.

So, in simple words it helps you to create your widgets for your desktop with Slint UI.
It, supprots rust as the backend, so as though there are not many batteries included
in the framework itself, everything can be brought to life from the dark arts of
rust.

## Installation.
You can use Spell by adding it to your project and finding more about it in the docs
(todo!, add URL).
```
cargo add Spell
```
Look at wuickstart for getting your first widget displaced.

## Why Slint ??
If you are cross the question of why not use any other gtk based widget system,
the next best doubt would be why Slint?

Slint because it is a simple yet powerful declarative language that is extremely
easy to learn (you can even get a sense of it in just 10 mins here). Secondly, unlike
other good UI kits, it just has awesome integration for rust. A competibility that
is hard to find.
```
```

## Inspiration
Just as I was thinking for a good enough system in **rust**, I simply couldn't find
any. Astal's official bindings were not ready and there was no other alternative.
I tried abstracting gtk myself for a crate but couldn't get a heck of it. Gtk is
good and tricky, I wanted to impliment some basic Shell logic, and Gtk is creating
full fledged apps. It was like taking a fly done with a bomb. So, I settled for just
a UI library(Slint) and only the code I need(rust). I found a exact same project
but it is not completed and maintained. So, I went out to cast the SPELL myself.
