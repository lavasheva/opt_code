namespace RefactoringExample;

// Парсер сохраняет исходный упрощённый CSV-контракт: разделитель — запятая,
// кавычки внутри значений не поддерживаются.
public static class CsvParser
{
    public static IEnumerable<Dictionary<string, string>> Parse(string[] lines)
    {
        string[] headers = lines[0].Split(',');
        foreach (string line in lines.Skip(1).Where(line => !string.IsNullOrEmpty(line)))
        {
            string[] values = line.Split(',');
            var record = new Dictionary<string, string>();
            for (int column = 0; column < headers.Length; column++)
                record[headers[column].Trim()] = values[column].Trim();
            yield return record;
        }
    }
}

public class DataImporter
{
    private readonly List<Dictionary<string, string>> records = new();
    private readonly TextWriter output;
    private readonly Func<IEnumerable<Dictionary<string, string>>, Task> save;
    // Диагностика доступна отдельно, консольный вывод остаётся совместимым.
    public Exception? LastError { get; private set; }

    public DataImporter(TextWriter? output = null,
        Func<IEnumerable<Dictionary<string, string>>, Task>? save = null)
    {
        this.output = output ?? Console.Out;
        this.save = save ?? SaveToDatabase;
    }

    public async Task ImportData(string filePath)
    {
        LastError = null;
        try
        {
            foreach (var record in CsvParser.Parse(File.ReadAllLines(filePath)))
            {
                if (IsProcessed(record)) records.Add(record);
                else output.WriteLine($"Skipping record with status: {record["status"]}");
            }
            await save(records);
        }
        catch (Exception error)
        {
            // Исходник подавлял исключения. Сохраняем этот контракт,
            // но даём вызывающему коду возможность узнать причину сбоя.
            LastError = error;
        }
    }

    public bool IsProcessed(Dictionary<string, string> record) => record["status"] == "processed";

    private async Task SaveToDatabase(IEnumerable<Dictionary<string, string>> data)
    {
        foreach (var record in data)
        {
            output.WriteLine($"Saving record: {record["id"]}");
            await Task.Delay(10);
        }
    }
}

public class OrderManager
{
    public void ProcessRecords(List<Dictionary<string, string>> records) => Console.WriteLine($"Processing {records.Count} records");
    public void ManageRecords(List<Dictionary<string, string>> records) => Console.WriteLine($"Managing {records.Count} records");
    // Старые имена оставлены как обёртки, чтобы существующие вызовы работали.
    public void Proc(List<Dictionary<string, string>> data) => ProcessRecords(data);
    public void Mgr(List<Dictionary<string, string>> data) => ManageRecords(data);
}

class Program
{
    static async Task Main(string[] args)
    {
        if (args.Contains("--test")) { await Tests.Run(); return; }
        const string file = "test_data.csv";
        File.WriteAllLines(file, new[] { "id,name,status", "1,Product A,processed", "2,Product B,pending", "3,Product C,processed", "4,Product D,rejected" });
        Console.WriteLine($"Created test CSV file: {file}");
        var importer = new DataImporter();
        await importer.ImportData(file);
        var data = new List<Dictionary<string, string>> {
            new() { ["id"] = "1", ["status"] = "processed" },
            new() { ["id"] = "2", ["status"] = "pending" }
        };
        var manager = new OrderManager();
        manager.Proc(data);
        manager.Mgr(data);
        foreach (var record in data)
            Console.WriteLine($"Record {record["id"]} is {(importer.IsProcessed(record) ? "" : "not ")}processed");
        Console.WriteLine("Application completed. Press any key to exit.");
        if (!args.Contains("--no-pause") && !Console.IsInputRedirected) Console.ReadKey();
    }
}
