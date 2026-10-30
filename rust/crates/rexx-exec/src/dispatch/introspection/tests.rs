use super::*;

#[test]
fn enhanced_metaclass_nil_entry_keeps_its_registered_method_through_gc() {
    let mut interp = Interp::new();
    let program = rexx_parse::parse_program(b"m=.method~new('PING', \"return 'pong'\")\nd=.directory~new\nd~ping=m\nreturn .class~enhanced(d, 'Remote')\n".to_vec()).expect("supported construction parses");
    let receiver = interp
        .run(program)
        .expect("class construction")
        .expect("class result");
    interp.roots.activity_mut().push_temp(receiver);
    let selection = receiver_selection(&mut interp, receiver).expect("real receiver selection");
    assert!(matches!(selection, Selection::ClassSide(class) if class == receiver));
    assert!(interp.own_method_entry(receiver, b"PING").is_none());
    let (scope, method) = lookup(&mut interp, selection, "PING").expect("enhanced entry");
    assert_eq!(scope, ObjRef::NIL);
    let executable = interp.executable_objects[&method];
    let owner = interp.class_of_value(receiver).expect("metaclass owner");
    for _ in 0..3 {
        interp.collect_now();
        assert!(interp.heap.get(executable).is_some());
        assert_eq!(interp.executable_objects.get(&method), Some(&executable));
        assert_eq!(
            lookup(&mut interp, selection, "PING"),
            Some((scope, method))
        );
        assert_eq!(
            method_object_for(&mut interp, selection, owner, b"PING", scope, method)
                .expect("existing nil-scoped Method"),
            executable
        );
    }
}
