/* PROCEDURE and PROCEDURE EXPOSE (4b Task 5, D9r).                          */
/*                                                                           */
/* Every value here is chosen so that the wrong answer prints something       */
/* different, which is not automatic: a callee setting a variable to the      */
/* value its caller already holds cannot distinguish isolation from sharing,  */
/* and an exposed variable whose value equals its own derived name cannot     */
/* distinguish exposure from non-exposure, because an unexposed unset read    */
/* yields the name. Nothing below holds a value equal to its own name.        */
/*                                                                           */
/* What each block would print if the implementation were wrong:              */
/*                                                                           */
/*   isolation   -- a callee sharing its caller's pool prints "callee-v",     */
/*                  not "caller-v".                                          */
/*   transitive  -- binding CEE's N to BEE's frame instead of chasing BEE's   */
/*                  own alias prints "from-a" two levels up. Binding every    */
/*                  exposed name of one PROCEDURE to a single target frame    */
/*                  gets M wrong instead: M is BEE's own local, so A must     */
/*                  still see "from-a-m" while BEE sees "set-by-cee-m".       */
/*   plural      -- EXPOSE (LIST) exposes both words of LIST's value; an      */
/*                  implementation treating the value as one name leaves      */
/*                  ALPHA and BETA in the caller untouched. GAMMA is the      */
/*                  control: it is never exposed and must not change.         */
/*   stem        -- EXPOSE aliases the caller's variable entry, so a tail     */
/*                  written in the callee is visible through the caller's     */
/*                  own stem.                                                 */

v = 'caller-v'
w = 'caller-w'
call sub
say 'isolation:' v w

n = 'from-a'
m = 'from-a-m'
call bee
say 'a sees:' n m

list = 'ALPHA BETA'
alpha = 'a-in-caller'
beta = 'b-in-caller'
gamma = 'g-in-caller'
call plural
say 'plural:' alpha beta gamma

st.1 = 'stem-kept'
call stemsub
say 'stem:' st.1
exit

sub: procedure expose w
v = 'callee-v'
w = 'callee-w'
return

bee: procedure expose n
m = 'from-bee-m'
call cee
say 'bee sees:' n m
return

cee: procedure expose n m
n = 'set-by-cee'
m = 'set-by-cee-m'
return

plural: procedure expose (list)
alpha = 'a-set'
beta = 'b-set'
gamma = 'g-set'
return

stemsub: procedure expose st.
st.1 = 'stem-changed'
return
