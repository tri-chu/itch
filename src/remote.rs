use std::env;

use git2::{
    Cred, CredentialType, FetchOptions, ProxyOptions, PushOptions, Remote, RemoteCallbacks,
};

use crate::{
    ctx::Ctx,
    error::{fail, Attempt, Maybe},
    print::show_warning,
};

fn get_remote_prefix() -> Maybe<String> {
    match env::var("ITCH_REMOTE_PREFIX") {
        Ok(v) => Ok(v),
        Err(env::VarError::NotPresent) => Ok(whoami::username() + "-"),
        Err(env::VarError::NotUnicode(_)) => fail("Non-unicode remote prefix specified"),
    }
}

fn setup_remote_callbacks(ctx: &Ctx) -> RemoteCallbacks<'_> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks
        .push_update_reference(|_, status| {
            if let Some(error_message) = status {
                fail(error_message)
            } else {
                Ok(())
            }
        })
        .credentials(|url, username_from_url, allowed_types| {
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                Cred::credential_helper(&ctx.repo.config()?, url, username_from_url)
            } else if allowed_types.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                    Some("git2023"),
                )
            } else {
                todo!("support for auth type: {allowed_types:?}");
            }
        });

    callbacks
}

fn setup_push_options(ctx: &Ctx) -> PushOptions<'_> {
    let mut options = PushOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn setup_fetch_options(ctx: &Ctx) -> FetchOptions<'_> {
    let mut options = FetchOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn get_remote(ctx: &Ctx) -> Maybe<Option<Remote>> {
    let remotes = ctx.repo.remotes()?;
    if remotes.is_empty() {
        return Ok(None);
    }
    if remotes.len() == 1 {
        return Ok(Some(ctx.repo.find_remote(remotes.get(0).unwrap())?));
    }
    let origin = ctx.repo.find_remote("origin");
    if let Ok(origin) = origin {
        return Ok(Some(origin));
    }
    fail("Unable to resolve default remote ('origin') out of multiple options")
}

pub fn push_branch(ctx: &Ctx, branch: &str) -> Attempt {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        let remote_prefix = get_remote_prefix()?;
        let refspec = format!(
            "+refs/heads/{}:refs/heads/{}{}",
            branch, remote_prefix, branch
        );
        remote.push(&[refspec], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}

pub fn pull_main(ctx: &Ctx) -> Attempt {
    match get_remote(ctx)? {
        None => Ok(()),
        Some(mut remote) => {
            remote.fetch(
                &["main"],
                Some(&mut setup_fetch_options(ctx)),
                Some("Fetch main"),
            )?;

            let local_main = ctx.repo.find_branch("main", git2::BranchType::Local)?;

            let mut local_ref = ctx
                .repo
                .find_branch("main", git2::BranchType::Local)?
                .into_reference();

            let remote_commit = ctx
                .repo
                .reference_to_annotated_commit(&local_main.upstream()?.into_reference())?;

            let analysis = ctx
                .repo
                .merge_analysis_for_ref(&local_ref, &[&remote_commit])?
                .0;

            if analysis.is_up_to_date() {
                Ok(())
            } else if analysis.is_fast_forward() {
                local_ref.set_target(remote_commit.id(), "Sync main")?;
                Ok(())
            } else {
                fail("Local diverges from remote.")
            }
        }
    }
}

pub fn push_main(ctx: &Ctx) -> Attempt {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        remote.push(&["refs/heads/main"], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}

pub fn try_push_branch(ctx: &Ctx, name: &str) {
    if let Err(e) = push_branch(ctx, name) {
        show_warning(
            ctx,
            &format!(
                "Failed to update remote; continuing anyway ({})",
                e.message()
            ),
        )
    }
}

pub fn try_push_main(ctx: &Ctx) {
    if let Err(e) = push_main(ctx) {
        show_warning(
            ctx,
            &format!("Failed to push remote; continuing anyway ({})", e.message()),
        )
    }
}

pub fn try_pull_main(ctx: &Ctx) {
    if let Err(e) = pull_main(ctx) {
        show_warning(
            ctx,
            &format!("Failed to pull remote; continuing anyway ({})", e.message()),
        )
    }
}
