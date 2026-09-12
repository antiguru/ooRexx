/* The file family's probe. Every `LIBRARY REXX` entry point this family owns
   is now built, so there is no refusal here to pin -- the test that runs these
   programs skips a family whose registry rows are all implemented, and holding
   one back so that a refusal existed would be a shell kept alive for a test.
   The program stays because every family owns one, and because the family's
   rows are what it would exercise again the moment one is deferred. */
say 'main'
say .k~probe

::class k

::method probe class external 'LIBRARY REXX file_exists'
