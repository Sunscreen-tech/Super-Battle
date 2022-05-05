use std::collections::HashMap;
use sunscreen::{
    types::{bfv::*, *},
    *,
};

#[derive(Clone)]
struct PlayerData {
    total_pl: Ciphertext,
    play_pl: Ciphertext,
    current_pl: Ciphertext,
}

struct Contract {
    pub player_data: HashMap<String, PlayerData>,
    runtime: Runtime,
    damage_fhe: CompiledFheProgram,
    level_up_fhe: CompiledFheProgram,
}

impl Contract {
    fn new() -> Self {
        #[fhe_program(scheme = "bfv")]
        fn deal_damage(current_pl: Cipher<Signed>, damage: Cipher<Signed>) -> Cipher<Signed> {
            current_pl - damage
        }

        #[fhe_program(scheme = "bfv")]
        fn level_up(total_pl: Cipher<Signed>, play_pl: Cipher<Signed>) -> Cipher<Signed> {
            let bet = total_pl - play_pl;

            bet * bet
        }

        let level_up_fhe = Compiler::with_fhe_program(level_up)
            .additional_noise_budget(50)
            .compile()
            .unwrap();

        let damage_fhe = Compiler::with_fhe_program(deal_damage)
            .with_params(&level_up_fhe.metadata.params)
            .compile()
            .unwrap();

        Self {
            player_data: HashMap::new(),
            runtime: Runtime::new(&level_up_fhe.metadata.params).unwrap(),
            level_up_fhe,
            damage_fhe,
        }
    }

    fn get_params(&self) -> &Params {
        &self.level_up_fhe.metadata.params
    }

    fn mint_nft(&mut self, public_key: &PublicKey, player_name: &str) {
        let data = PlayerData {
            total_pl: self.runtime.encrypt(Signed::from(50), public_key).unwrap(),
            play_pl: self.runtime.encrypt(Signed::from(0), public_key).unwrap(),
            current_pl: self.runtime.encrypt(Signed::from(0), public_key).unwrap(),
        };

        self.player_data.insert(player_name.to_owned(), data);
    }

    fn battle(&mut self, alice: &Player, bob: &Player) {
        let mut alice_data = self.player_data.get(&alice.name).unwrap().clone();
        let mut bob_data = self.player_data.get(&bob.name).unwrap().clone();

        alice_data.play_pl = alice.get_play_pl(&alice_data.total_pl);
        alice_data.current_pl = alice_data.play_pl.clone();
        bob_data.play_pl = bob.get_play_pl(&bob_data.total_pl);
        bob_data.current_pl = bob_data.play_pl.clone();

        loop {
            let alice_damage = alice.get_damage(&alice_data.current_pl, &bob.public_key);
            let bob_damage = bob.get_damage(&bob_data.current_pl, &alice.public_key);

            alice_data.current_pl = self
                .runtime
                .run(
                    &self.damage_fhe,
                    vec![alice_data.current_pl.clone(), bob_damage],
                    &alice.public_key,
                )
                .unwrap()[0]
                .clone();

            bob_data.current_pl = self
                .runtime
                .run(
                    &self.damage_fhe,
                    vec![bob_data.current_pl.clone(), alice_damage],
                    &bob.public_key,
                )
                .unwrap()[0]
                .clone();

            let alice_state = alice.get_state(&alice_data.current_pl);
            let bob_state = bob.get_state(&bob_data.current_pl);

            if alice_state == PlayerState::Dead && bob_state == PlayerState::Dead {
                println!("Both contestants are vanquished!");
                break;
            } else if alice_state == PlayerState::Dead {
                bob_data.total_pl = self.level_up(
                    bob_data.total_pl,
                    bob_data.play_pl.clone(),
                    &bob.public_key,
                );
                self.player_data.insert(bob.name.to_owned(), bob_data);

                println!("{} has been defeated in combat!", alice.name);
                break;
            } else if bob_state == PlayerState::Dead {
                alice_data.total_pl =
                    self.level_up(alice_data.total_pl, alice_data.play_pl.clone(), &alice.public_key);
                self.player_data.insert(alice.name.to_owned(), alice_data);

                println!("{} has been defeated in combat!", bob.name);
                break;
            }

            if alice_state == PlayerState::Reborn {
                println!("{} has emerged from the ashes and is reborn!", alice.name);
                alice_data.current_pl = alice_data.play_pl.clone();
            }

            if bob_state == PlayerState::Reborn {
                println!("{} has emerged from the ashes and is reborn!", bob.name);
                bob_data.current_pl = bob_data.play_pl.clone();
            }
        }
    }

    fn level_up(
        &self,
        total_pl: Ciphertext,
        play_pl: Ciphertext,
        public_key: &PublicKey,
    ) -> Ciphertext {
        self.runtime
            .run(&self.level_up_fhe, vec![total_pl, play_pl], public_key)
            .unwrap()[0]
            .clone()
    }
}

fn prompt_for_int(prompt: &str) -> i64 {
    let int_val;

    println!("{}", prompt);

    loop {
        let mut line = String::default();
        std::io::stdin().read_line(&mut line).unwrap();

        match line.trim().parse::<i64>() {
            Ok(v) => {
                int_val = v;
                break;
            }
            _ => {
                println!("Not an integer. Try again.");
            }
        };
    }

    int_val
}

struct Player {
    pub public_key: PublicKey,
    pub name: String,
    private_key: PrivateKey,
    runtime: Runtime,
}

impl Player {
    pub fn new(name: &str, params: &Params) -> Self {
        let runtime = Runtime::new(params).unwrap();
        let (public, private) = runtime.generate_keys().unwrap();

        Self {
            public_key: public,
            private_key: private,
            runtime,
            name: name.to_owned(),
        }
    }

    pub fn get_play_pl(&self, total_pl: &Ciphertext) -> Ciphertext {
        let total_pl: Signed = self.runtime.decrypt(total_pl, &self.private_key).unwrap();
        let total_pl: i64 = total_pl.into();

        let mut play_pl;

        loop {
            play_pl = prompt_for_int(&format!(
                "{}, how much power do you prepare for battle (shh, don't tell anyone, {} is max)???",
                self.name,
                total_pl
            ));

            if play_pl > 0 && play_pl <= total_pl {
                break;
            } else {
                println!("Foolish mortal! You are not strong enough!");
            }
        }

        // Need to prove play_pl > 0 && play_pl <= total_pl
        self.runtime
            .encrypt(Signed::from(play_pl), &self.public_key)
            .unwrap()
    }

    pub fn get_damage(&self, current_pl: &Ciphertext, opponent_key: &PublicKey) -> Ciphertext {
        let current_pl: Signed = self.runtime.decrypt(current_pl, &self.private_key).unwrap();
        let current_pl: i64 = current_pl.into();

        let mut damage;

        loop {
            damage = prompt_for_int(&format!(
                "{}, how much damage do you deal (shh, don't tell anyone, {} is max)???",
                self.name,
                current_pl
            ));

            if damage > 0 && damage <= current_pl {
                break;
            } else {
                println!("Foolish mortal! You are not strong enough!");
            }
        }

        self.runtime
            .encrypt(Signed::from(damage), opponent_key)
            .unwrap()
    }

    pub fn get_state(&self, current_pl: &Ciphertext) -> PlayerState {
        let current_pl: Signed = self.runtime.decrypt(current_pl, &self.private_key).unwrap();
        let current_pl: i64 = current_pl.into();

        if current_pl < -10 {
            PlayerState::Reborn
        } else if current_pl <= 0 {
            PlayerState::Dead
        } else {
            PlayerState::Alive
        }
    }
}

#[derive(PartialEq)]
enum PlayerState {
    Alive, // ZKP
    Dead,
    Reborn, // ZKP
}

fn main() {
    let mut contract = Contract::new();

    let alice = Player::new("Alice", contract.get_params());
    let bob = Player::new("Bob", contract.get_params());

    contract.mint_nft(&alice.public_key, &alice.name);
    contract.mint_nft(&bob.public_key, &bob.name);

    loop {
        contract.battle(&alice, &bob);
    }
}
