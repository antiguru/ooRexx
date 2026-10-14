# Classes and their defining packages are never collected (D59)

Source: Phase 9b1 Task 5 report, concern 3. Phase 5b's D59 ruled classes are never collected (D60:
cheap). With packages now collectable, a package that defines a class stays alive through the class;
the oracle collects both. Licensed for now as collection timing under D59. Revisit: collecting
classes needs the registry and method-dict pruning paths (expunge already exists), the class weight
charge (9b1 Task 7) and a witness with a WeakReference to such a package.
