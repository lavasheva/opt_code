using System.Diagnostics;
using System.Globalization;

static class Program
{
    public static int Original(int n) => n <= 1 ? n : unchecked(Original(n - 1) + Original(n - 2));

    public static int Optimized(int n)
    {
        if (n <= 1) return n;
        int previous = 0, current = 1;
        for (int i = 2; i <= n; i++)
            (previous, current) = (current, unchecked(previous + current));
        return current;
    }

    static void Main(string[] args)
    {
        for (int n = -3; n <= 30; n++)
            if (Original(n) != Optimized(n)) throw new Exception($"Mismatch at {n}");
        if (Optimized(35) != 9227465 || Optimized(47) != -1323752223)
            throw new Exception("Known values or int overflow changed");
        Console.WriteLine("PASS: 36 checks (including int overflow)");
        if (args.Contains("--test")) return;
        Directory.CreateDirectory("results");
        using var csv = new StreamWriter("results/measurements.csv");
        csv.WriteLine("N,before_ms,after_ms,speedup");
        // Прогрев JIT перед измерениями. Для быстрого метода — пакет вызовов.
        for (int i = 0; i < 10000; i++) Optimized(35);
        Original(20);
        foreach (int n in new[] { 20, 25, 30, 32, 35 })
        {
            double before = MedianTime(() => Original(n), 1);
            double after = MedianTime(() => Optimized(n), 100000);
            string row = FormattableString.Invariant($"{n},{before:F8},{after:F8},{before / after:F2}");
            csv.WriteLine(row);
            Console.WriteLine(row);
        }
        Console.WriteLine($"Fibonacci(35) = {Optimized(35)}");
    }

    static double MedianTime(Func<int> calculate, int repetitions)
    {
        var measurements = new List<double>();
        for (int sample = 0; sample < 5; sample++)
        {
            var timer = Stopwatch.StartNew();
            int value = 0;
            for (int i = 0; i < repetitions; i++) value ^= calculate();
            timer.Stop();
            GC.KeepAlive(value);
            measurements.Add(timer.Elapsed.TotalMilliseconds / repetitions);
        }
        measurements.Sort();
        return measurements[2];
    }
}
