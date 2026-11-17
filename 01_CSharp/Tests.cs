namespace RefactoringExample;

static class Tests
{
    public static async Task Run()
    {
        string file = Path.GetTempFileName();
        try
        {
            var saved = new List<string>();
            var log = new StringWriter();
            var importer = new DataImporter(log, rows => {
                saved.Add(string.Join(",", rows.Select(row => row["id"])));
                return Task.CompletedTask;
            });
            File.WriteAllLines(file, new[] { "id,status", " 1 , processed ", "2,pending", "", "3,processed" });
            await importer.ImportData(file);
            Check(saved.SequenceEqual(new[] { "1,3" }), "filter and trim");
            Check(log.ToString().Trim() == "Skipping record with status: pending", "skip log");
            await importer.ImportData(file);
            Check(saved[1] == "1,3,1,3", "repeated import keeps original accumulation");
            File.WriteAllText(file, "");
            await importer.ImportData(file);
            Check(importer.LastError != null && saved.Count == 2, "empty file: suppressed error, no save");
            File.WriteAllLines(file, new[] { "id,status", "1" });
            await importer.ImportData(file);
            Check(importer.LastError != null, "short record");
            Console.WriteLine("PASS: 5 checks");
        }
        finally { File.Delete(file); }
    }

    static void Check(bool condition, string name)
    {
        if (!condition) throw new Exception("FAIL: " + name);
    }
}
