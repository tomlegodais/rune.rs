#[macros::on_npc(op = Op1, npc_id = 2)]
async fn talk_to_man() {
    npc_dialogue!("Hello there, {}! Can I help you with something?", player.username);
    player_dialogue!("Maybe. I'm just having a look around.");
    npc_dialogue!("Well, you've come to the right person. I know this area like the back of my hand.");

    dialogue_tree! {
        dialogue_choice! {
            "Who are you?" => goto!(who),
            "What's going on around here?" => goto!(whats_going_on),
            "Can you give me any advice?" => goto!(advice),
            "Goodbye." => goto!(bye),
        }

        dialogue!(who) {
            player_dialogue!("Who exactly are you?");
            npc_dialogue!("Just a man. Nothing more, nothing less.");
            npc_dialogue!("I've lived here my whole life. Seen a lot of people pass through.");
            goto!(root);
        }

        dialogue!(whats_going_on) {
            player_dialogue!("What's going on around here?");
            npc_dialogue!("Not much, to be honest. Same as always.");
            npc_dialogue!("Though I did hear some strange noises from the forest last night. Probably nothing.");

            dialogue_choice! {
                "That sounds worrying." => goto!(worrying),
                "I'll look into it." => goto!(look_into_it),
                "Probably nothing." => goto!(root),
            }
        }

        dialogue!(worrying) {
            player_dialogue!("That sounds really worrying!");
            npc_dialogue!("Aye, I thought so too. But what can you do?");

            dialogue_choice! {
                "I could check it out for you." => goto!(look_into_it),
                "Fair enough." => goto!(root),
            }
        }

        dialogue!(look_into_it) {
            player_dialogue!("I'll go take a look for you.");
            npc_dialogue!("Would you really? Be careful out there.");
            npc_dialogue!("The forest isn't what it used to be. Stick to the paths if you can.");
            goto!(done);
        }

        dialogue!(advice) {
            player_dialogue!("Can you give me any advice?");

            dialogue_choice! {
                "About combat?" => goto!(advice_combat),
                "About making money?" => goto!(advice_money),
                "Never mind." => goto!(root),
            }
        }

        dialogue!(advice_combat) {
            npc_dialogue!("Always keep some food on you. You'd be surprised how many don't.");
            npc_dialogue!("And don't pick fights you can't win. No shame in running.");
            goto!(root);
        }

        dialogue!(advice_money) {
            npc_dialogue!("There's good money in fishing, if you've got the patience.");
            npc_dialogue!("Or try your hand at smithing. People always need armour.");
            goto!(root);
        }

        dialogue!(bye) {
            player_dialogue!("Goodbye!");
            npc_dialogue!("Safe travels, {}. Come back any time.", player.username);
            goto!(done);
        }
    }

    npc_dialogue!("One more thing before you go...");
    npc_dialogue!("If you find anything strange, come tell me about it.");
    player_dialogue!("I will. Thanks for the tip.");

    dialogue_tree! {
        dialogue_choice! {
            "Where exactly in the forest?" => goto!(where_forest),
            "What should I look out for?" => goto!(look_out),
            "I'll head out now." => goto!(done),
        }

        dialogue!(where_forest) {
            npc_dialogue!("North side, near the old ruins. That's where the sounds were coming from.");
            goto!(done);
        }

        dialogue!(look_out) {
            npc_dialogue!("Couldn't say for sure. Didn't sound like any animal I've heard before.");
            npc_dialogue!("Could be goblins, could be worse. Just keep your wits about you.");
            goto!(done);
        }
    }

    npc_dialogue!("Good luck out there, {}.", player.username);
}
