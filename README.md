# DSC Bot

The DSC Bot is designed to assist with moderation across server by having a shared banlist, shared warnings, and synced role information, along with a strike system and wordfilter.

## Features
- Moderation
    - Strikes (tracked per server)
    - Advisories (Strikes, but global)
    - Global ban list
    - Wordfilter to remove message containing banend words
    - Ability to view reasons for strikes, advisories, and bans.
- Verification
    - Age category information for YPT
    - Easy system for verifying roles

## Running
Make sure that you have the latest stable [Rust](https://rustup.rs). Make a copy of `.env.example` and name it `.env`, filling out the variables as appropriate.

## Building
- Install latest stable [Rust](https://rustup.rs)
- Clone this repository with `git clone https://github.com/Discord-Scout-Council/DSC-Bot.git`
- Go into the new project directory and run `cargo build`

## Submitting patches
Patches can be submitted by forking and making a pull request. [Instructions] (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request-from-a-fork)

