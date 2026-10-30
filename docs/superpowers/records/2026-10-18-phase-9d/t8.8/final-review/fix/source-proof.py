import hashlib,json,pathlib,subprocess
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records=root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix'
def show(commit,path):return subprocess.check_output(['git','show',commit+':'+path],cwd=root)
def digest(data):return hashlib.sha256(data).hexdigest()
base='4fe3e24d640702cddaaca25b87efc93012c1928f'
stream='rust/crates/rexx-exec/src/dispatch/stream.rs'
old=show(base,stream).decode()
first=old[old.index('/// `query_exists`'):old.index('/// The errno a failed open reports')]
second=old[old.index('/// `stream_query_position`'):old.index('/// `stream_close`')]
moved=show('f7d6e8d5e','rust/crates/rexx-exec/src/dispatch/stream/query.rs').decode()
expected=(first+second).replace('pub(super) fn','pub(in crate::dispatch) fn')
actual=moved[moved.index('/// `query_exists`'):]
assert actual.strip()==expected.strip()
body='rust/crates/rexx-core/src/body.rs'
old=show('21af23d3f^',body).decode()
chunk=old[old.index('/// How many written bytes a buffered stream holds'):old.index('/// A `Stream`\'s own state')]
new=show('21af23d3f','rust/crates/rexx-core/src/body/open_file.rs').decode()
assert new[new.index('/// How many written bytes a buffered stream holds'):]==chunk
assert '#[test]' not in chunk and '```' not in chunk
changed=subprocess.check_output(['git','diff','--name-only',base,'HEAD','--','rust'],cwd=root,text=True).splitlines()
source=[p for p in changed if '/src/' in p]
assert set(source)=={body,'rust/crates/rexx-core/src/body/open_file.rs',stream,'rust/crates/rexx-exec/src/dispatch/stream/query.rs','rust/crates/rexx-exec/src/dispatch/hash/relation.rs','rust/crates/rexx-exec/src/dispatch/hash.rs','rust/crates/rexx-exec/src/dispatch/hash/removal.rs'}
assert 'assert!(size_of::<StreamState>() <= 232)' in (root/body).read_text()
hash_path='rust/crates/rexx-exec/src/dispatch/hash.rs'
removal='rust/crates/rexx-exec/src/dispatch/hash/removal.rs'
old=show('d7091492c^',hash_path).decode()
chunk=old[old.index('/// `HashCollection::emptyRexx`'):old.index('\nfn native_hash_has_index')].strip()
new=show('d7091492c',removal).decode()
start=new.index('/// `HashCollection::emptyRexx`')
assert new[start:new.index('pub(super) fn native_hash_remove_item',start)].strip()==chunk.replace('fn native_hash_empty','pub(super) fn native_hash_empty')
relation='rust/crates/rexx-exec/src/dispatch/hash/relation.rs'
old=show(base,relation).decode();new=(root/relation).read_text()
marker='/// `RelationClass::hasItemRexx'
assert old[:old.index(marker)]==new[:new.index(marker)]
def function(text,name):
 start=text.index('fn '+name+'(')
 return text[start:text.index('\n}\n',start)+3]
assert function(old,'native_relation_remove_item')==function(new,'native_relation_remove_item')
for name in ['take_at','native_hash_remove_item']:
 assert function(show(base,removal).decode(),name)==function((root/removal).read_text(),name)
result=dict(empty_move_equal_after_visibility=True,prior_removal_allat_supplier_bodies_unchanged=True,base=base,head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),stream_move_equal_after_visibility=True,core_move_byte_equal=True,moved_tests_or_doctest_blocks=False,changed_runtime=source,source_hashes={p:digest((root/p).read_bytes()) for p in source},test_name_changes={'added':['owned_stdout_position_is_empty_before_open_and_after_close','owned_stdout_aliases_keep_their_buffered_positions'],'removed':[]})
(records/'source-proof.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
