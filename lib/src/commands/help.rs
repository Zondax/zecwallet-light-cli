use zcash_primitives::consensus;
use crate::commands::{Command, get_commands};
use crate::lightclient::LightClient;

pub struct HelpCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for HelpCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("List all available commands");
        h.push("Usage:");
        h.push("help [command_name]");
        h.push("");
        h.push("If no \"command_name\" is specified, a list of all available commands is returned");
        h.push("Example:");
        h.push("help send");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Lists all available commands".to_string()
    }

    fn exec(
        &self,
        args: &[&str],
        _client: &LightClient<P>,
    ) -> String {
        let mut responses = vec![];

        // Print a list of all commands
        match args.len() {
            0 => {
                responses.push(format!("Available commands:"));
                get_commands::<P>()
                    .iter()
                    .for_each(|(cmd, obj)| {
                        responses.push(format!("{} - {}", cmd, obj.short_help()));
                    });

                responses.join("\n")
            },
            1 => match get_commands::<P>().get(args[0]) {
                Some(obj) => obj.help(),
                None => format!("Command {} not found", args[0]),
            },
            _ => Command::<P>::help(self),
        }
    }
}
