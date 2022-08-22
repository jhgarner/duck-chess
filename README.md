# Duck Chess

Duck chess is a chess variant created here: https://duckchess.com/

This is an implementation of Duck Chess features (a)synchronous multiplayer. To
play, just create an account. You'll be able to see open games to join and be
able to create your own open games. Once you join an open game, it'll show up on
your main menu.

## Rules of Duck Chess

See the link above, but basically there are three rules on top of normal chess:

1. The duck is impassable. You may not end your turn on the duck and you may not
   move through the duck, although the horse can jump over it.
2. After making a normal chess move, you must move the duck to a new empty
   square.
3. Check and checkmate don't exist. You win the game by taking your opponents
   king. You may perform actions that put your king in check.


## TODO

Frontend:
1. Add notifications
2. Use Tauri to create a native frontend
3. Add a "local" game button to UI

Backend:
1. Actually track sessions
2. Add notifications
