// Java 测试代码 — 调用生成的 UniFFI JNI 绑定
// 编译：javac -cp examples/simple/generated/java TestSimple.java
// 运行：java -Djava.library.path=examples/simple/generated/rust-glue/target/debug -cp examples/simple/generated/java;. TestSimple

import com.example.uniffi.simple;
import com.example.uniffi.simple.Calculator;
import com.example.uniffi.simple.MyData;
import com.example.uniffi.simple.Color;
import com.example.uniffi.simple.Shape;

public class TestSimple {
    public static void main(String[] args) {
        System.out.println("=== UniFFI JNI 绑定测试 ===");
        System.out.println();

        // ---- 1. 测试顶层函数 ----
        System.out.println("--- 1. 顶层函数 ---");
        long sum = simple.add(10, 20);
        System.out.println("add(10, 20) = " + sum);
        assert sum == 30 : "add 失败";

        long product = simple.multiply(6, 7);
        System.out.println("multiply(6, 7) = " + product);
        assert product == 42 : "multiply 失败";

        String greeting = simple.greet("世界");
        System.out.println("greet(\"世界\") = " + greeting);
        assert greeting.equals("Hello, 世界!") : "greet 失败";
        System.out.println("  顶层函数：全部通过 ✅");
        System.out.println();

        // ---- 2. 测试 Calculator 对象 ----
        System.out.println("--- 2. Calculator 对象 ---");
        Calculator calc = Calculator.newNew(100);
        System.out.println("new Calculator(100) → handle ok");

        long val = calc.add(50);
        System.out.println("calc.add(50) = " + val);
        assert val == 150 : "add 失败";

        val = calc.subtract(30);
        System.out.println("calc.subtract(30) = " + val);
        assert val == 120 : "subtract 失败";

        val = calc.getValue();
        System.out.println("calc.getValue() = " + val);
        assert val == 120 : "getValue 失败";

        MyData input = new MyData(5, "测试数据");
        MyData output = calc.processData(input);
        System.out.println("calc.processData(MyData(5, \"测试数据\")) = MyData("
            + output.getValue() + ", \"" + output.getLabel() + "\")");
        assert output.getValue() == 125 : "processData value 失败";
        assert output.getLabel().equals("processed: 测试数据") : "processData label 失败";

        calc.close();
        System.out.println("calc.close() → ok");
        System.out.println("  Calculator：全部通过 ✅");
        System.out.println();

        // ---- 3. 测试 Record (MyData) ----
        System.out.println("--- 3. Record (MyData) ---");
        MyData data = new MyData(42, "答案");
        System.out.println("new MyData(42, \"答案\") → ok");
        System.out.println("  getValue() = " + data.getValue());
        System.out.println("  getLabel() = " + data.getLabel());
        System.out.println("  toString() = " + data.toString());
        assert data.getValue() == 42;
        assert data.getLabel().equals("答案");
        System.out.println("  Record：通过 ✅");
        System.out.println();

        // ---- 4. 测试 Enum (Color) ----
        System.out.println("--- 4. Enum (Color) ---");
        Color red = new Color.Red();
        Color green = new Color.Green();
        Color blue = new Color.Blue();
        System.out.println("Color.Red    = " + red);
        System.out.println("Color.Green  = " + green);
        System.out.println("Color.Blue   = " + blue);
        System.out.println("  Enum：通过 ✅");
        System.out.println();

        // ---- 5. 测试 Enum (Shape) ----
        System.out.println("--- 5. Enum (Shape) ---");
        Shape circle = new Shape.Circle(5.0);
        Shape rect = new Shape.Rectangle(3.0, 4.0);
        Shape point = new Shape.Point();
        System.out.println("Shape.Circle(5.0)      = " + circle);
        System.out.println("Shape.Rectangle(3,4)   = " + rect);
        System.out.println("Shape.Point()          = " + point);
        System.out.println("  Enum：通过 ✅");
        System.out.println();

        System.out.println("========================================");
        System.out.println("   全部测试通过！🎉");
        System.out.println("========================================");
    }
}
