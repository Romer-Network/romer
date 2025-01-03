# main.rs

so main.rs will start getting pretty verbose if we start putting this all in here. 

[ ] Now we can Start the runtime via executor.start in Commonware nomenclature Oracle is a mechanism to register authorized peers. So peers that are not explicitly authorized will be blocked by commonware-p2p. This is interesting as I believe this means that only the Genesis node can setup authorized peers. So what is commonware's plans for Nodes to join permissionlessly? Or can I as a new node self register myself via the Oracle::register(). Perhaps we can use this to make sure that when a peer wants to join they have to prove latency over a week or so and we can track them? There is also a counter everytime the validator set changes

[ ] The network needs to register channels. So in the log example, there are channels for sender and resolver
[ ] We now can initialize storage. We need to give it a partition. So we can create blocks for now
[ ] Then we go to an Actor.rs file in order to return application, supervisor and mailbox. Automaton, Relay and Committer come from Mailbox



