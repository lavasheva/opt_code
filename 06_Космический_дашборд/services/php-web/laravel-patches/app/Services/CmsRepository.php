<?php
namespace App\Services;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class CmsRepository
{
    public function page(string $slug): ?object
    {
        return Cache::remember('cms:'.$slug, 60, fn() =>
            DB::table('cms_pages')->where('slug', $slug)->first());
    }

    public function safeHtml(string $html): string
    {
        $config = \HTMLPurifier_Config::createDefault();
        $config->set('HTML.Allowed', 'h2,h3,p,br,strong,b,em,i,ul,ol,li,a[href|title]');
        $config->set('URI.AllowedSchemes', ['http'=>true, 'https'=>true]);
        $config->set('Cache.SerializerPath', storage_path('framework/cache/data'));
        return (new \HTMLPurifier($config))->purify($html);
    }
}
