/* PACKAGE and PROTECTED, at every sending position one package can offer.
 *
 * Both answer here and both answered before the access scopes were built, so
 * neither row is a new agreement. What they are is the guard on the direction
 * the corpus can see: a send that starts refusing shows up as a wrong exit
 * status, where a refusal that starts answering does not. PACKAGE compares
 * the package the method was translated in against the package the send is
 * written in, and everything in one file is one package, so every row below
 * is the allowing side.
 *
 * The refusing side of PACKAGE needs a caller in a second package, which
 * needs ::REQUIRES, and no program here can reach it.
 *
 * PROTECTED is the one scope the oracle asks a security manager about.
 * Nothing installs one, so the question has a single answer and these rows
 * cannot tell the question from its absence.
 *
 * Measured, rc 0.
 */

say 'pkg-outside' .K~pkg
say 'pkg-inside' .K~viaSelf
say 'pkg-sibling' .S~poke
say 'pkg-routine' r()
say 'prot-outside' .K~prot
say 'prot-inside' .K~viaProt
say 'attr-package' .K~att
say 'done'

::CLASS K
::METHOD pkg CLASS PACKAGE
  return 'package scope'
::METHOD viaSelf CLASS
  return self~pkg
::METHOD prot CLASS PROTECTED
  return 'protected'
::METHOD viaProt CLASS
  return self~prot
::ATTRIBUTE att CLASS GET PACKAGE
  return 'package attribute'

::CLASS S
::METHOD poke CLASS
  return .K~pkg

::ROUTINE r
  return .K~pkg
