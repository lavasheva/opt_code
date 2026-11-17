<?php
namespace App\Services;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class SpaceClient
{
    public function get(string $path, array $query = []): array
    {
        $base = rtrim(getenv('RUST_BASE') ?: 'http://rust_iss:3000', '/');
        // Исключения и ok=false не кэшируем.
        $key = 'space:'.hash('sha256', $base.$path.json_encode($query));
        if ($cached = Cache::get($key)) return $cached;
        try {
            $response = Http::connectTimeout(2)->timeout(5)->get($base.$path, $query);
            $response->throw();
            $data = $response->json();
            if (!is_array($data)) throw new \RuntimeException('Invalid JSON');
            if (($data['ok'] ?? true) !== false) Cache::put($key, $data, 10);
            return $data;
        } catch (\Throwable $error) {
            $trace = (string) Str::uuid();
            Log::warning('Space upstream failed', ['trace_id'=>$trace]);
            return ['ok'=>false,'error'=>['code'=>'UPSTREAM_ERROR',
                'message'=>'Данные временно недоступны','trace_id'=>$trace]];
        }
    }
}
