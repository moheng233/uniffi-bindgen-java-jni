// Test for uniffi-example-arithmetic fixture.
// Validates basic top-level function bindings.
//
// Compile: javac -cp <generated-java-dir> -d <classes-dir> tests/scripts/TestArithmetic.java
// Run (Windows):     java -Djava.library.path=<rust-glue>/target/debug -cp <generated-java-dir>;<classes-dir> TestArithmetic
// Run (Linux/macOS): java -Djava.library.path=<rust-glue>/target/debug -cp <generated-java-dir>:<classes-dir> TestArithmetic

import uniffi.fixtures.arithmetic;

public class TestArithmetic {
    public static void main(String[] args) {
        System.out.println("=== Arithmetic Test ===");

        // div: basic division
        long result = arithmetic.div(8L, 4L);
        System.out.println("div(8, 4) = " + result);
        assert result == 2L : "div(8,4) should be 2";

        // equal: equality check
        assert arithmetic.equal(2L, 2L) : "equal(2,2) should be true";
        assert arithmetic.equal(4L, 4L) : "equal(4,4) should be true";
        assert !arithmetic.equal(2L, 4L) : "equal(2,4) should be false";
        assert !arithmetic.equal(4L, 8L) : "equal(4,8) should be false";

        System.out.println("All arithmetic tests passed!");
    }
}
