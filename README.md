# Duck Chess

Duck chess is a chess variant created here: https://duckchess.com/

You can play this version at https://duck.ohea.xyz

This is an implementation of Duck Chess featuring (a)synchronous multiplayer. To
play, just create an account. You'll be able to see open games to join and be
able to create your own open games. Once you join an open game, it'll show up on
your main menu.

This web app supports notifications. If you enable them, the app will alert you
when it's your turn. This feature isn't supported yet on IOS, but works very
well on Android and mostly on desktop.

## Rules of Duck Chess

See the link above, but basically there are three rules on top of normal chess:

1. The duck is impassable. You may not end your turn on the duck and you may not
   move through the duck. The only exception is the horse which can jump over it.
2. After making a normal chess move, you must move the duck to a new empty
   square.
3. Check and checkmate don't exist. You win the game by taking your opponent's
   king. You may perform actions that put your king in check.


## TODO

Frontend:
1. Use Tauri to create a native frontend
2. Allow offline play

Backend:
1. Allow anonymous users
