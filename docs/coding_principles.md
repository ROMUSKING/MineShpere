# Coding Principles

This document outlines key coding principles to be followed in this project, inspired by NASA's safety-critical guidelines and Jonathan Blow's "Tiger Style" philosophy.

## NASA's Power of 10 Rules (Safety-Critical C/C++)

Developed by Gerard J. Holzmann at NASA/JPL, these rules focus on reliability and verifiability.

1.  **Simple Control Flow**: Restrict code to very simple control flow constructs. Do not use `goto`, `setjmp`, `longjmp`, or recursion.
2.  **Fixed Loop Bounds**: All loops must have a fixed upper bound. It must be trivially possible for a checking tool to prove statically that a preset upper bound on the number of iterations cannot be exceeded.
3.  **No Dynamic Memory Allocation**: Do not use dynamic memory allocation after initialization.
4.  **Short Functions**: No function should be longer than what can be printed on a single sheet of paper (typically ~60 lines).
5.  **Assertion Density**: The code's assertion density should average to minimally two assertions per function. Assertions must be side-effect free.
6.  **Smallest Scope**: Declare all data objects at the smallest possible level of scope.
7.  **Check Return Values**: Each calling function must check the return value of nonvoid functions, and each called function must check the validity of all parameters provided by the caller.
8.  **Limited Preprocessor Use**: The use of the preprocessor must be limited to the inclusion of header files and simple macro definitions. Token pasting, variable argument lists (ellipses), and recursive macro calls are not allowed.
9.  **Limited Pointer Use**: Limit pointer use to a single dereference, and do not use function pointers.
10. **Pedantic Compilation**: Compile with all possible warnings active; all warnings should then be addressed before the release of the software. Code must be checked daily with at least one static source code analyzer.

## TigerBeetle's "Tiger Style"

A rigorous coding philosophy for distributed financial databases, emphasizing safety, performance, and developer experience.

1.  **Safety First**: "Zero technical debt" policy. Do things right the first time.
2.  **Performance**: Optimize critical resources (network, disk, memory, CPU) from the start.
3.  **Static Allocation**: Use static memory allocation to prevent fragmentation and OOM errors.
4.  **Deterministic Simulation Testing (DST)**: Test code in a simulated environment (VOPR) to uncover logical errors.
5.  **Fail-Fast**: Use pervasive assertions. Assertion failures should crash the program with detailed logs.
6.  **Explicit Control Flow**: Avoid recursion and complex control structures.
7.  **Strict Limits**: Set explicit upper bounds for loops and data structures.
8.  **Small Functions**: Keep functions concise (ideally under 70 lines).
9.  **Data Integrity**: Zero trust in data read from disk/network (checksums). Append-only immutability.
10. **Developer Experience**: Prioritize readable, easy-to-work-with code.


## Tiger Style Coding (Jonathan Blow)

"Tiger Style" emphasizes deep understanding, efficiency, and aesthetic quality in code.

1.  **Aesthetics of Code**: View programming decisions as aesthetic choices. Prioritize clarity, elegance, and the interaction of elements.
2.  **Aversion to Complexity**: Avoid unnecessary complexity. Reject "demo-driven development" and over-engineered solutions. Solve the actual problem directly.
3.  **Direct Solutions**: Many problems are not inherently difficult but are made so by over-complication. Aim for the most direct path to the solution.
4.  **High Standards & Iteration**: Hold high personal standards. Be willing to rewrite and refine code multiple times to achieve the right solution.
5.  **Specific over General**: Favor specific code over overly general solutions. General code often comes with a cost in complexity and performance.
6.  **Productivity & Skill**: Tools and languages should empower skilled developers to be extraordinarily productive.
7.  **Pragmatism**: Be willing to deviate from "best practices" (like rigid adherence to Clean Code) if a different approach is more effective for the specific problem at hand.
