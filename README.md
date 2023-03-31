# About
A simple Discord application (slash command) that overlays a ChaosDAO ring on top of user-provided image
based on the user's role.

# How it works
1. The user types `/ring` in a channel.
2. A form with a field for attachment is displayed.
3. The user adds a file and sends.
4. The app responds with progress message.
5. A few seconds later the app responds with an image.

# Requirements
The application requires the following environment variables to be set:
- Discord token - a token obtained when the discord application was linked to a bot in the Discord's dashboard
  - The way Apps work they have to be "linked" to a bot. You have to add a bot under your Application but you don't need to assign it any permissions at all.
  - To generate an invitation link for the app, go to "URL Generator" under "OAuth2" and generate a url with just the `applications.commands` scope selected.
  - Use this link to add the app to any of your servers.
- Role IDs - actual Discord user Roles
  - One for each of _Frens_, _Regulars_ and _DAOists_
- Image Paths - the ring images used as overlays
  - One file for each of _Frens_, _Regulars_ and _DAOists_

These names of the environment variables (and example values) are:
```shell
CHAOSRING_DAOISTS=/path/to/daoists_ring.png
CHAOSRING_FRENS=/path/to/frens_ring.png
CHAOSRING_REGULARS=/path/to/regulars_ring.png
DAO_ROLE_DAOIST=<role_id>
DAO_ROLE_FREN=<role_id>
DAO_ROLE_REGULAR=<role_id>
DISCORD_TOKEN=<token>
```
## Docker image
```shell
docker build --platform linux/x86_64 -t chaosbot .
```
