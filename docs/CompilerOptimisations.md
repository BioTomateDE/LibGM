## Ideas for GML Compiler Optimisations
**Changes and additions are welcome!!!**
**Just message `@farming.simulator` on Discord.**

The numbers represent the optimisations levels the user will be able to choose from.

0. No optimisations
    - Nothing is changed.
1. Slight optimisations
    - Dead code removal
      - Unreachable if statements
      - Redundant variable overwriting
      - Unused variables
    - Usage of `PushImmediate` when possible
      - Converts int pushes within -32768 and 32767
      - Saves space (4 bytes)
   - Evaluate constant expressions
       - Precompute mathematical expressions like `6 * 7 - 41` --> `1`
       - Floats will maybe cause issues??? they should be standardized on every platform though, right?
       - Loop unrolling (hard to implement though)
2. Medium optimisations
   - Convert self variables to local when possible
     - Need to account for the dynamic variable accessor functions
     - I don't know if this is actually faster
   - Inline function (script) calls
     - Basically just paste the script code, replacing the call
     - Removes the need for `push` and `conv.v` instructions
     - No more context switching overhead
     - Very situational (when should this actually be done?)
3. Heavy optimisations
    - Convert local variables to stack values
        - Most prominent example: for loop iterators
        - Will produce `_temp_local_var_67` when decompiled
        - Need to account for the dynamic variable accessor functions

Note that these are *just ideas*. Most of these are probably too hard to implement
and don't offer a big performance increase. But ever since I started using Rust, 
I just have to think about optimising every little bit of code for my perfectionism.
