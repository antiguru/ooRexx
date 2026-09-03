/* The Stem object: its string value, its class, and the aliasing a bare stem
   read hands out (5c Task 7).

   What each block would print if a constructed stem were an ordinary
   instance of .Stem, which is what this crate built and backed out once:

     value    -- `a Stem` in place of the default, at rc 0 on both sides,
                 because the send would fall through to Object~STRING. That
                 is the silent wrong answer this program exists to catch;
                 nothing else in any harness goes red for it.
     name     -- a stem with no default renders as its own name, not as an
                 empty string and not as `a Stem`. ~objectName is `a Stem`
                 on both, so the two answers part here and the pair is what
                 pins which is which.
     (unlabelled) -- the bare SAYs of a constructed stem are the silent
                 shape itself, compared byte for byte: `BARE.` and an empty
                 line, where a plain instance prints `a Stem` at rc 0.
     new      -- .Stem~new's argument IS the string value, and an omitted
                 one is the null string, not the class name.
     alias    -- a bare stem read hands back the live object, so a write
                 through either name is visible through both. A read that
                 snapshotted the value would make `al.` a fresh stem
                 defaulting to `dflt`, and every `alias` read would answer
                 `dflt` -- the tail writes would be landing in a different
                 stem. The last line is the other half: a bare assignment
                 REPLACES the variable's object and leaves the alias on the
                 old one. */

s. = 'dflt'
o = s.
say 'value' o o~string o~objectName o~defaultName
say 'value' o~class~id o~isA(.Stem) o~isA(.Object) o~isNil

t.1 = 'tail'
n = t.
say 'name' n n~string n~objectName n~class~id

say .Stem~new('BARE.')
say .Stem~new
say 'new' '[' || .Stem~new || ']' '[' || .Stem~new('FOO.') || ']' '[' || .Stem~new(5) || ']'
say 'new' .Stem~new~class~id .Stem~new('FOO.')~string .Stem~new~string~length

s.1 = 'one'
al. = o
say 'alias' al.1 al.
s.1 = 'two'
say 'alias' al.1
al.2 = 'through-the-alias'
say 'alias' s.2

made = .Stem~new('N.')
cc. = made
cc.1 = 'written'
say 'alias' cc.1 cc. made

s. = 'replaced'
say 'alias' al.1 o s.1
