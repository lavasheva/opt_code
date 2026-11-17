<?php
namespace Tests\Feature;
use Tests\TestCase;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Cache;
use App\Services\UpstreamClient;

class SpaceClientTest extends TestCase
{
    public function test_success_is_cached(): void
    {
        Cache::flush();
        Http::fake(['example.test/*'=>Http::response(['value'=>42])]);
        $client = app(UpstreamClient::class);
        $this->assertSame(['value'=>42], $client->get('https://example.test/data'));
        $client->get('https://example.test/data');
        Http::assertSentCount(1);
    }
    public function test_errors_are_not_cached(): void
    {
        Cache::flush();
        Http::fake(['example.test/*'=>Http::response([],403)]);
        $client = app(UpstreamClient::class);
        $this->assertSame('UPSTREAM_403', $client->get('https://example.test/data')['error']['code']);
        $client->get('https://example.test/data');
        Http::assertSentCount(2);
    }
    public function test_bad_coordinates(): void
    {
        $result = app(\App\Services\AstroService::class)->events(['lat'=>100]);
        $this->assertFalse($result['ok']);
        $this->assertSame('INVALID_COORDINATES', $result['error']['code']);
    }
    public function test_cms_keeps_formatting_and_removes_active_content(): void
    {
        $html = app(\App\Services\CmsRepository::class)->safeHtml(
            '<h3>Title</h3><p onclick="alert(1)">Text</p><script>alert(1)</script><a href="javascript:alert(1)">link</a>');
        $this->assertStringContainsString('<h3>Title</h3>', $html);
        $this->assertStringContainsString('<p>Text</p>', $html);
        $this->assertStringNotContainsString('<script', $html);
        $this->assertStringNotContainsString('onclick', $html);
        $this->assertStringNotContainsString('javascript:', $html);
    }
    public function test_jwst_failure_preserves_http_200_error_contract(): void
    {
        Cache::flush();
        Http::fake(['*'=>Http::response([],403)]);
        $this->getJson('/api/jwst/feed')->assertOk()
            ->assertJsonPath('ok', false)->assertJsonPath('error.code', 'UPSTREAM_403');
    }
}
