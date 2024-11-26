1. Compile the c code with a Makefile, integrate that into build.rs.
2. Enable interpreting by running aves_interpreter, following the TODOs in that file.
3. Complete fill_in_samples.sh, generating the other forms of IR and the expected output files.
4. Modify test_roundtrip.sh to test the `.aves_text` forms of the programs as well, making sure they result in the same bytecode. Also, have it check the output of interpreting both forms. Rename the script to `runtests.sh`.
5. Work on all the remaining TODOs.