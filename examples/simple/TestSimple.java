// Java 测试代码 — 调用生成的 UniFFI JNI 绑定
// 编译：javac -cp examples/simple/generated/java TestSimple.java
// 运行：java -Djava.library.path=examples/simple/generated/rust-glue/target/debug -cp examples/simple/generated/java;. TestSimple

import com.example.uniffi.simple;
import com.example.uniffi.simple.Calculator;
import com.example.uniffi.simple.CalculatorListener;
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

        // ---- 6. 测试 Callback Interface (CalculatorListener) ----
        System.out.println("--- 6. Callback Interface (CalculatorListener) ---");

        // 6a. 创建一个 Java 实现并使用 FfiConverter 注册
        CalculatorListener listener = new CalculatorListener() {
            private String lastOperation = "";
            private long lastValue = 0;
            private int onCalculationCalls = 0;
            private int onResetCalls = 0;

            @Override
            public void onCalculation(String operation, long value) {
                lastOperation = operation;
                lastValue = value;
                onCalculationCalls++;
                System.out.println("  [callback] onCalculation(\"" + operation + "\", " + value + ")");
            }

            @Override
            public void onReset() {
                onResetCalls++;
                System.out.println("  [callback] onReset()");
            }
        };

        // 6b. 注册回调（通过 FfiConverter.lower 写入 HandleMap）
        long handle = simple.FfiConverterCalculatorListener.INSTANCE.lower(listener);
        System.out.println("FfiConverter.lower(listener) → handle = " + handle);
        assert handle > 0 : "回调注册失败：handle 应为正数";

        // 6c. 通过 FfiConverter.lift 取回
        CalculatorListener lifted = simple.FfiConverterCalculatorListener.INSTANCE.lift(handle);
        assert lifted == listener : "回调 lift 失败：应返回同一个实例";
        System.out.println("FfiConverter.lift(handle) → " + (lifted == listener ? "同一个实例 ✅" : "失败 ❌"));

        // 6d. 测试 write/read（ByteBuffer 序列化）
        java.nio.ByteBuffer buf = java.nio.ByteBuffer.allocateDirect(8);
        buf.order(java.nio.ByteOrder.BIG_ENDIAN);
        simple.FfiConverterCalculatorListener.INSTANCE.write(listener, buf);
        buf.flip();
        CalculatorListener readBack = simple.FfiConverterCalculatorListener.INSTANCE.read(buf);
        assert readBack == listener : "回调 read 失败：应返回同一个实例";
        System.out.println("FfiConverter.write() + read() 往返 → 通过 ✅");

        // 6e. 测试 allocationSize
        int allocSize = simple.FfiConverterCalculatorListener.INSTANCE.allocationSize(listener);
        assert allocSize == 8 : "allocationSize 应为 8";
        System.out.println("allocationSize = " + allocSize + " → 通过 ✅");

        // 6f. 测试通过 registerCalculatorListener() default 方法注册
        CalculatorListener listener2 = new CalculatorListener() {
            @Override public void onCalculation(String op, long val) {}
            @Override public void onReset() {}
        };
        long handle2 = listener2.registerCalculatorListener();
        System.out.println("listener.registerCalculatorListener() → handle = " + handle2);
        assert handle2 > 0 && handle2 != handle : "registerCalculatorListener 失败";

        // 6g. 直接调用静态回调分发方法（模拟 Rust→JNI→Java 路径）
        System.out.println("直接调用 callback 分发方法:");
        simple.callbackCalculatorListener_onCalculation(handle, "add", 42);
        simple.callbackCalculatorListener_onCalculation(handle, "multiply", 99);
        simple.callbackCalculatorListener_onReset(handle);

        // 6h. 验证无效 handle 会抛异常
        try {
            simple.callbackCalculatorListener_onCalculation(99999, "bad", 0);
            System.err.println("  ❌ 无效 handle 应抛异常！");
            assert false : "应抛异常";
        } catch (RuntimeException e) {
            System.out.println("  无效 handle → 正确抛异常: " + e.getMessage().substring(0, Math.min(40, e.getMessage().length())) + "...");
        }

        System.out.println("  Callback Interface：全部通过 ✅");
        System.out.println();

        System.out.println("========================================");
        System.out.println("   全部测试通过！🎉");
        System.out.println("========================================");
    }
}
