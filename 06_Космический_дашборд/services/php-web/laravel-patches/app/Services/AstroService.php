<?php
namespace App\Services;
class AstroService
{
    public function __construct(private UpstreamClient $client) {}
    public function events(array $input): array
    {
        $lat = filter_var($input['lat'] ?? 55.7558, FILTER_VALIDATE_FLOAT);
        $lon = filter_var($input['lon'] ?? 37.6176, FILTER_VALIDATE_FLOAT);
        if ($lat === false || $lon === false || abs($lat)>90 || abs($lon)>180)
            return $this->client->error('INVALID_COORDINATES');
        $id = getenv('ASTRO_APP_ID') ?: '';
        $secret = getenv('ASTRO_APP_SECRET') ?: '';
        if (!$id || !$secret) return $this->client->error('MISSING_CREDENTIALS');
        $days = max(1, min(30, (int)($input['days'] ?? 7)));
        return $this->client->get('https://api.astronomyapi.com/api/v2/bodies/events', [
            'latitude'=>$lat,'longitude'=>$lon,'from'=>now('UTC')->toDateString(),
            'to'=>now('UTC')->addDays($days)->toDateString()
        ], ['Authorization'=>'Basic '.base64_encode($id.':'.$secret)]);
    }
}
