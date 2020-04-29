# Campmaster Constantine

[![builds.sr.ht status](https://builds.sr.ht/~muirrum/campmasterconstantine/.build.yml.svg)](https://builds.sr.ht/~muirrum/campmasterconstantine/.build.yml?)

The Campmaster is designed to be a general-purpose Discord bot, that brings a message points/leveling system and a strike system to make moderation easier.

## Features
- Points gain based on messages
    - Levels based on slow exponential (1.3x) cost to next level
    - Level-based role rewarads*
- Moderation
    - Strikes as warnings
    - Modification of a strike's reason
    - Withdrawal of strikes
    - Automatic banning/kicking of users based on strike count*
    - Global and local message filter
    - Moderation log for strikes and wordfilter matches
- Question of the Day
    - Suggestions
    - Automatic ping of a role

## Running
Make sure that you have the latest stable [Rust](https://rustup.rs). Make a copy of `.env.example` and name it `.env`, filling out the variables as appropriate.

Run `cargo install campmaster-constantine` to download and install the latest stable version of the bot. Then, run `campmaster-constantine` in the same directory as your `.env` file.

## Building
- Install latest stable [Rust](https://rustup.rs)
- Clone this repository with `git clone https://git.sr.ht/~muirrum/campmasterconstantine`
- Go into the new project directory and run `cargo build`

## Submitting patches
Send an email with the patch file to `~muirrum/campmaster-constantine-devel@lists.sr.ht`, preferably following the instructions [here](https://git-send-email.io).