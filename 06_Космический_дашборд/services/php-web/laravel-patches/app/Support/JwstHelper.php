<?php

namespace App\Support;

final class JwstHelper
{
    private string $host;
    private string $key;
    private ?string $email;

    public function __construct()
    {
        $this->host  = rtrim(getenv('JWST_HOST') ?: 'https://api.jwstapi.com', '/');
        $this->key   = getenv('JWST_API_KEY') ?: '';
        $this->email = getenv('JWST_EMAIL') ?: null;
    }

    public function get(string $path, array $qs = []): array
    {
        $headers = ['x-api-key'=>$this->key];
        if ($this->email) $headers['email'] = $this->email;
        return app(\App\Services\UpstreamClient::class)->get(
            $this->host.'/'.ltrim($path, '/'), $qs, $headers);
    }

    /** ищем первую пригодную картинку в произвольной структуре */
    public static function pickImageUrl(array $v): ?string
    {
        $stack = [$v];
        while ($stack) {
            $cur = array_pop($stack);
            foreach ($cur as $k => $val) {
                if (is_string($val) && preg_match('~^https?://.*\.(?:jpg|jpeg|png)$~i', $val)) return $val;
                if (is_array($val)) $stack[] = $val;
            }
        }
        return null;
    }
}
