use log::error;
use serenity::{client::Context, model::id::GuildId};

use super::super::explicit_command_list::COMMAND_LIST;

pub enum UpdateStatus {
    FirstSetting,
    UpdateAvailable(Vec<String>),
    LatestVersion,
    FailedtoLoad,
}

//Version을 파라미터로 업데이트하게끔 변경
pub async fn check_updates(ctx: &Context, gid: GuildId) -> UpdateStatus {
    match gid.get_application_commands(&ctx.http).await {
        Ok(cmds) => {
            let mut guild_cmds_list = cmds
                .clone()
                .into_iter()
                .map(|v| v.name)
                .collect::<Vec<String>>();
            let mut server_cmds_list = Vec::new();
            for scmds in COMMAND_LIST.commands.values() {
                server_cmds_list.push(scmds.name());
            }
            server_cmds_list.push("update".to_string());

            guild_cmds_list.sort();
            server_cmds_list.sort();

            return if cmds.len() == 1 {
                UpdateStatus::FirstSetting
            //개수로 판단하는거는 문제가 있음. 기존꺼 삭제하고 하나 추가하면..
            } else if guild_cmds_list == server_cmds_list {
                UpdateStatus::LatestVersion
            } else {
                //아직 등록되지 않은 명령어 목록을 parameter로 전달
                let mut unassigned_commands = Vec::new();
                let cmdname_lists = cmds.iter().map(|c| c.name.clone()).collect::<Vec<String>>();
                for (cmdname, _) in COMMAND_LIST.commands.iter() {
                    if !cmdname_lists.contains(&cmdname.to_string()) {
                        unassigned_commands.push(cmdname.to_string());
                    }
                }
                UpdateStatus::UpdateAvailable(unassigned_commands)
            };
        }
        Err(why) => {
            error!(
                "Failed to get application data from {}. \nwhy: {:#?}",
                gid, why
            );
            UpdateStatus::FailedtoLoad
        }
    }
}
