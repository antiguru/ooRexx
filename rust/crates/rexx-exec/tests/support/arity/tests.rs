use super::{NONE, Receiver, program};

fn plain(setup: &str) -> Receiver {
    Receiver {
        setup: setup.to_string(),
        wrapper: NONE.to_string(),
        directives: NONE.to_string(),
    }
}

/// The program text `corpus/collection-arity.tsv` was derived with, spelled
/// out rather than described.
#[test]
fn a_receiver_with_no_wrapper_and_no_directives_writes_the_original_program() {
    assert_eq!(
        program(
            &plain("ka = 'k1' | r = .Table~new | r[ka] = 'v1'"),
            "at",
            "ka",
            false
        ),
        "ka = 'k1'\n\
         r = .Table~new\n\
         r[ka] = 'v1'\n\
         say 'SETUP-OK'\n\
         signal on syntax name oops\n\
         r~'at'(ka)\n\
         say 'SENT'\n\
         exit\n\
         oops:\n\
         say 'SYNTAX' condition('O')~code\n\
         exit 0\n"
    );
}

/// `--` for the argument list is a send with none, not a send of `--`.
#[test]
fn an_empty_argument_list_sends_no_parentheses() {
    assert!(
        program(&plain("r = .Table~new"), "items", NONE, false)
            .contains("\nr~'items'\nsay 'SENT'\n")
    );
}

/// Comparing values assigns the send and prints the answer, and reads the
/// one code a method returning no result raises as a completed send.
#[test]
fn comparing_values_assigns_the_send_and_prints_the_answer() {
    assert_eq!(
        program(&plain("r = .Table~new"), "items", NONE, true),
        "r = .Table~new\n\
         say 'SETUP-OK'\n\
         signal on syntax name oops\n\
         vv = r~'items'\n\
         say 'SENT'\n\
         say 'VALUE' vv~string\n\
         exit\n\
         oops:\n\
         vc = rc || '.' || condition('E')\n\
         if vc = '91.999' then do\n\
         say 'SENT'\n\
         say 'NORESULT'\n\
         exit 0\n\
         end\n\
         say 'SYNTAX' vc\n\
         exit 0\n"
    );
}

/// The wrapper moves the setup and the send inside the call and leaves the
/// trap in the caller, which is where the condition arrives.
#[test]
fn the_call_wrapper_puts_the_setup_and_the_send_inside_the_routine() {
    let text = program(
        &Receiver {
            setup: "r = .context".to_string(),
            wrapper: "call".to_string(),
            directives: NONE.to_string(),
        },
        "name",
        NONE,
        false,
    );
    assert_eq!(
        text,
        "signal on syntax name oops\n\
         call probe 'a1', 'a2'\n\
         say 'SENT'\n\
         exit\n\
         oops:\n\
         say 'SYNTAX' condition('O')~code\n\
         exit 0\n\
         probe:\n\
         r = .context\n\
         say 'SETUP-OK'\n\
         r~'name'\n\
         return\n"
    );
}

/// Directives are appended after the program, one per `|`-separated part.
#[test]
fn directives_are_appended_after_the_program() {
    let text = program(
        &Receiver {
            setup: "r = .K".to_string(),
            wrapper: NONE.to_string(),
            directives: "::class K subclass Object | ::method MM | return 2".to_string(),
        },
        "id",
        NONE,
        false,
    );
    assert!(
        text.ends_with("::class K subclass Object\n::method MM\nreturn 2\n"),
        "{text}"
    );
    assert!(text.starts_with("r = .K\nsay 'SETUP-OK'\n"), "{text}");
}
