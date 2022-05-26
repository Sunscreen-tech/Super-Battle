use sunscreen::{
    types::{bfv::Signed, Cipher},
    *,
};

use std::collections::HashMap;

#[derive(Clone)]
pub struct Nft {
    pub total_pl: Ciphertext,
    pub play_pl: Ciphertext,
    pub current_pl: Ciphertext,
}

pub struct Contract {
    pub runtime: Runtime,
    pub deal_damage_fhe: CompiledFheProgram,
    pub level_up_fhe: CompiledFheProgram,
    pub state: HashMap<String, Nft>,
}

const PIKACHU: &str = r"
quu..__
 $$$b  `---.__
  '$$b        `--.                          ___.---uuudP
   `$$b           `.__.------.__     __.---'      $$$$'              .
     '$b          -'            `-.-'            $$$'              .'|
       '.                                       d$'             _.'  |
         `.   /                              ...'             .'     |
           `./                           ..::-'            _.'       |
            /                         .:::-'            .-'         .'
           :                          ::''\          _.'            |
          .' .-.             .-.           `.      .'               |
          : /'$$|           .@'$\           `.   .'              _.-'
         .'|$u$$|          |$$,$$|           |  <            _.-'
         | `:$$:'          :$$$$$:           `.  `.       .-'
         :                  `'--'             |    `-.     \
        :##.       ==             .###.       `.      `.    `\
        |##:                      :###:        |        >     >
        |#'     `..'`..'          `###'        x:      /     /
         \                                   xXX|     /    ./
          \                                xXXX'|    /   ./
          /`-.                                  `.  /   /
         :    `-  ...........,                   | /  .'
         |         ``:::::::'       .            |<    `.
         |             ```          |           x| \ `.:``.
         |                         .'    /'   xXX|  `:`M`M':.
         |    |                    ;    /:' xXXX'|  -'MMMMM:'
         `.  .'                   :    /:'       |-'MMMM.-'
          |  |                   .'   /'        .'MMM.-'
          `'`'                   :  ,'          |MMM<
            |                     `'            |tbap\
             \                                  :MM.-'
              \                 |              .''
               \.               `.            /
                /     .:::::::.. :           /
               |     .:::::::::::`.         /
               |   .:::------------\       /
              /   .''               >::'  /
              `',:                 :    .'
                                   `:.:'
";

const PICOLO: &str = r"

               _.---..._
            ./^         ^-._
          ./^C===.         ^\.   /\
         .|'     \\        _ ^|.^.|
    ___.--'_     ( )  .      ./ /||
   /.---^T\      ,     |     / /|||
  C'   ._`|  ._ /  __,-/    / /-,||
       \ \/    ;  /O  / _    |) )|,
        i \./^O\./_,-^/^    ,;-^,'
         \ |`--/ ..-^^      |_-^
          `|  \^-           /|:
           i.  .--         / '|.
            i   =='       /'  |\._
          _./`._        //    |.  ^-ooo.._
   _.oo../'  |  ^-.__./X/   . `|    |#######b
  d####     |'      ^^^^   /   |    _\#######
  #####b ^^^^^^^^--. ...--^--^^^^^^^_.d######
  ######b._         Y            _.d#########
  ##########b._     |        _.d#############
";

const GOKU: &str = r"
                   _
                   \'-._ _.--'~~'--._
                    \   '            ^.    ___
                    /                  \.-~_.-~
             .-----'     /\/'\ /~-._      /
            /  __      _/\-.__\L_.-/\     '-.
           /.-'  \    ( ` \_o>'<o_/  \  .--._\
          /'      \    \:     '     :/_/     '`
                  /  /\ '\    ~    /~'
                  \ I  \/]'-._ _.-'[
               ___ \|___/ ./    l   \___   ___
          .--v~   'v` ( `-.__   __.-' ) ~v'   ~v--.
       .-{   |     :   \_    '~'    _/   :     |   }-.
      /   \  |           ~-.,___,.-~           |  /   \
     ]     \ |                                 | /     [
     /\     \|     :                     :     |/     /\
    /  ^._  _K.___,^                     ^.___,K_  _.^  \
   /   /  '~/  '\                           /'  \~'  \   \
  /   /    /     \ _          :          _ /     \    \   \
.^--./    /       Y___________l___________Y       \    \.--^.
[    \   /        |        [/    ]        |        \   /    ]
|     'v'         l________[____/]________j  -Row   }r'     /
}------t          /                       \       /`-.     /
|      |         Y                         Y     /    '-._/
}-----v'         |         :               |     7-.     /
|   |_|          |         l               |    / . '-._/
l  .[_]          :          \              :  r[]/_.  /
 \_____]                     '--.             '-.____/

";

impl Contract {
    pub fn new() -> Self {
        #[fhe_program(scheme = "bfv")]
        fn level_up(total_pl: Cipher<Signed>, play_pl: Cipher<Signed>) -> Cipher<Signed> {
            let tmp = total_pl - play_pl;

            tmp * tmp
        }

        #[fhe_program(scheme = "bfv")]
        fn deal_damage(current_pl: Cipher<Signed>, damage: Cipher<Signed>) -> Cipher<Signed> {
            current_pl - damage
        }

        let level_up_fhe = Compiler::with_fhe_program(level_up)
            .additional_noise_budget(100)
            .compile()
            .unwrap();

        let deal_damage_fhe = Compiler::with_fhe_program(deal_damage)
            .with_params(&level_up_fhe.metadata.params)
            .compile()
            .unwrap();

        let runtime = Runtime::new(&level_up_fhe.metadata.params).unwrap();

        Self {
            runtime,
            deal_damage_fhe,
            level_up_fhe,
            state: HashMap::new(),
        }
    }

    pub fn get_params(&self) -> &Params {
        &self.level_up_fhe.metadata.params
    }

    pub fn mint_nft(&mut self, name: &str, public_key: &PublicKey) {
        let total_pl = self.runtime.encrypt(Signed::from(50), public_key).unwrap();
        let current_pl = total_pl.clone();
        let play_pl = total_pl.clone();

        let nft = Nft {
            total_pl,
            play_pl,
            current_pl,
        };

        self.state.insert(name.to_owned(), nft);
    }

    pub fn deal_damage(&self, current_pl: Ciphertext, damage: Ciphertext, public_key: &PublicKey) -> Ciphertext {
        let result = self.runtime.run(
            &self.deal_damage_fhe,
            vec![current_pl, damage],
            public_key
        ).unwrap();

        result[0].clone()
    }

    pub fn level_up(&self, total_pl: Ciphertext, play_pl: Ciphertext, public_key: &PublicKey) -> Ciphertext {
        let result = self.runtime.run(
            &self.level_up_fhe,
            vec![total_pl, play_pl],
            public_key
        ).unwrap();

        result[0].clone()
    }

    fn battle(&mut self, alice: &Player, bob: &Player) {
        let mut alice_data = self.state.get(&alice.name).unwrap().clone();
        let mut bob_data = self.state.get(&bob.name).unwrap().clone();

        alice_data.play_pl = alice.get_play_pl(&alice_data.total_pl);
        alice_data.current_pl = alice_data.play_pl.clone();
        bob_data.play_pl = bob.get_play_pl(&bob_data.total_pl);
        bob_data.current_pl = bob_data.play_pl.clone();

        loop {
            let alice_damage = alice.get_damage(&alice_data.current_pl, &bob.public_key);
            let bob_damage = bob.get_damage(&bob_data.current_pl, &alice.public_key);

            alice_data.current_pl =
                self.deal_damage(alice_data.current_pl.clone(), bob_damage, &alice.public_key);

            bob_data.current_pl =
                self.deal_damage(bob_data.current_pl.clone(), alice_damage, &bob.public_key);

            let alice_state = alice.get_state(&alice_data.current_pl);
            let bob_state = bob.get_state(&bob_data.current_pl);

            if alice_state == PlayerState::Dead && bob_state == PlayerState::Dead {
                println!("{}", PIKACHU);
                println!("Both contestants are vanquished!");
                break;
            } else if alice_state == PlayerState::Dead {
                bob_data.total_pl =
                    self.level_up(bob_data.total_pl, bob_data.play_pl.clone(), &bob.public_key);
                self.state.insert(bob.name.to_owned(), bob_data);

                println!("{}", PICOLO);
                println!("{} has been defeated in combat!", alice.name);
                break;
            } else if bob_state == PlayerState::Dead {
                alice_data.total_pl = self.level_up(
                    alice_data.total_pl,
                    alice_data.play_pl.clone(),
                    &alice.public_key,
                );
                self.state.insert(alice.name.to_owned(), alice_data);

                println!("{}", GOKU);
                println!("{} has been defeated in combat!", bob.name);
                break;
            }

            if alice_state == PlayerState::Reborn {
                println!("{}", PICOLO);
                println!("{} has emerged from the ashes and is reborn!", alice.name);
                alice_data.current_pl = alice_data.play_pl.clone();
            }

            if bob_state == PlayerState::Reborn {
                println!("{}", GOKU);
                println!("{} has emerged from the ashes and is reborn!", bob.name);
                bob_data.current_pl = bob_data.play_pl.clone();
            }
        }
    }
}

pub struct Player {
    pub public_key: PublicKey,
    private_key: PrivateKey,
    pub runtime: Runtime,
    pub name: String,
}

impl Player {
    pub fn new(name: &str, params: &Params) -> Self {
        let runtime = Runtime::new(params).unwrap();

        let (public_key, private_key) = runtime.generate_keys().unwrap();

        Self {
            public_key,
            private_key,
            runtime,
            name: name.to_owned(),
        }
    }

    pub fn get_play_pl(&self, total_pl: &Ciphertext) -> Ciphertext {
        let total_pl: Signed = self.runtime.decrypt(total_pl, &self.private_key).unwrap();
        let total_pl: i64 = total_pl.into();

        if total_pl == 0 {
            println!("{} is no match for basic arithmetic! Leveling up has killed you! exiting.", self.name);
            std::process::exit(0);
        }

        let play_pl = Player::prompt_for_int(
            &format!("{}, choose your play power! You maximum is {}, but keep it secret!", self.name, total_pl),
            0,
            total_pl,
            "Foolish mortal! You are not strong enough!",
        );

        self.runtime.encrypt(Signed::from(play_pl), &self.public_key).unwrap()
    }

    pub fn get_damage(&self, current_pl: &Ciphertext, public_key: &PublicKey) -> Ciphertext {
        let current_pl: Signed = self.runtime.decrypt(current_pl, &self.private_key).unwrap();
        let current_pl: i64 = current_pl.into();

        let damage = Player::prompt_for_int(
            &format!("{}, choose your damage! You maximum is {}, but keep it secret!", self.name, current_pl),
            0,
            current_pl,
            "Foolish mortal! You are not strong enough!",
        );

        self.runtime.encrypt(Signed::from(damage), &public_key).unwrap()
    }

    pub fn get_state(&self, current_pl: &Ciphertext) -> PlayerState {
        let current_pl: Signed = self.runtime.decrypt(current_pl, &self.private_key).unwrap();
        let current_pl: i64 = current_pl.into();
        
        if current_pl > 0 {
            PlayerState::Alive
        } else if current_pl < -10 {
            PlayerState::Reborn
        } else {
            PlayerState::Dead
        }
    }

    fn prompt_for_int(prompt: &str, min: i64, max: i64, out_of_range_message: &str) -> i64 {
        let int_val;

        use std::io::Write;

        // Blank the terminal
        println!("{}", prompt);

        loop {
            let mut line = String::default();
            std::io::stdin().read_line(&mut line).unwrap();

            match line.trim().parse::<i64>() {
                Ok(v) => {
                    if v > max || v < min {
                        println!("{}", out_of_range_message);
                    } else {
                        int_val = v;
                        break;
                    }
                }
                _ => {
                    println!("Not an integer. Try again.");
                }
            };
        }

        for i in 0..5 {
            print!("{esc}c", esc = 27 as char);
            println!("{}", 5 - i);
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        print!("{esc}c", esc = 27 as char);
        let _ = std::io::stdout().flush();

        int_val
    }
}

#[derive(PartialEq)]
pub enum PlayerState {
    Alive,
    Dead,
    Reborn,
}

fn main() {
    print!("{esc}c", esc = 27 as char);

    let mut contract = Contract::new();

    let alice = Player::new("Alice", contract.get_params());
    let bob = Player::new("Bob", contract.get_params());

    contract.mint_nft(&alice.name, &alice.public_key);
    contract.mint_nft(&bob.name, &bob.public_key);

    loop {
        contract.battle(&alice, &bob);
    }
}
