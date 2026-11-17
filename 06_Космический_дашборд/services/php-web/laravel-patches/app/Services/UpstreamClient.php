<?php
namespace App\Services;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class UpstreamClient
{
    public function get(string $url, array $query = [], array $headers = []): array
    {
        $key = 'upstream:'.hash('sha256', $url.json_encode($query).json_encode($headers));
        if ($value = Cache::get($key)) return $value;
        try {
            $response = Http::withHeaders($headers)->connectTimeout(3)->timeout(8)
                ->withUserAgent('StudentSpaceDashboard/1.0')->get($url, $query);
            if ($response->failed()) return $this->error('UPSTREAM_'.$response->status());
            $value = $response->json();
            if (!is_array($value)) return $this->error('INVALID_JSON');
            Cache::put($key, $value, 30);
            return $value;
        } catch (\Throwable $error) { return $this->error('UPSTREAM_TIMEOUT'); }
    }

    public function error(string $code): array
    {
        $trace = (string) Str::uuid();
        Log::warning('Upstream failed', ['code'=>$code, 'trace_id'=>$trace]);
        return ['ok'=>false,'error'=>['code'=>$code,'message'=>'Данные временно недоступны','trace_id'=>$trace]];
    }
}
