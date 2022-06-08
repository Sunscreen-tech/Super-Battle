# Super-Battle
This repo contains the Alice and Bob Super Battle game demoed at ETH Prague. The game leverages the Sunscreen Fully Homomorphic Encryption (FHE) compiler to hide players' power levels. Throught the power of FHE, the game loop is able to deal damage to players and level them up on victory without ever decrypting their stats! The game plays as follows:

Power level (PL) is effectively the player's health and the maximum damage they can deal.

1. A service (e.g. a smart contract) initializes each players' `total_pl` to `100` encrypted under their respective public keys.
2. At the start of battle, each player chooses a `play_pl`: `0 < play_pl < total_pl`, encrypts it with their public key, shares it, and publishes a zero-knowlege proof (ZKP, not implemented) to show their play_pl is valid. This helps obfuscate players' maximum health during battle.
3. The service assigns `current_pl = play_pl` (i.e. it copies the ciphertext).
4. Each round, players choose `damage` to deal to their opponent between `0 < damage < current_pl`. They encrypt this damage under their opponent's public key, publish the ciphertext, and produce a ZKP (not implemented) to show their choice is valid.
5. The trusted service uses homomorphic addition to subtract the damage from each player's `current_pl`.
6. Each player must prove (not implemented) either their `current_pl > 0` or `current_pl < -10`. In the latter case, the service resets their `current_pl` to the `play_pl` ciphertext, resurrecting them to full health. This punishes overkilling your opponent.
7. If one player remains alive the service levels them up with the following formula run homomorphically: `total_pl = (total_pl - play_pl)^2`. Dead player(s) retain their old `total_pl`.
8. Play continues with more powerful players! Goto 2.
