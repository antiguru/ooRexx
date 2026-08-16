/* The package's own class table is consulted BEFORE .environment, which is
 * the order this task exists to get right. Phase 5a Task 6.
 *
 * ::class array declares a class whose id is ARRAY -- upcased, because the
 * directive names it with a symbol -- and the environment already holds a
 * class registered under ARRAY whose id is Array. The two render
 * differently, so which one .ARRAY answers is observable: the package's.
 *
 * .DIRECTORY is the control. This file declares no class of that name, so it
 * reaches the environment and renders with the environment's own spelling --
 * without it, a build that consulted only the package table would satisfy
 * the line above and answer nothing at all here.
 *
 * Measured, rc 0.
 */

say .ARRAY
say .DIRECTORY

::class array
