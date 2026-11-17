<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Support\JwstHelper;

class DashboardController extends Controller
{
    public function index()
    {
        // минимум: карта МКС и пустые контейнеры, JWST-галерея подтянется через /api/jwst/feed
        $iss = app(\App\Services\SpaceClient::class)->get('/last');
        $trend = []; // фронт сам заберёт /api/iss/trend (через nginx прокси)
        $cms = app(\App\Services\CmsRepository::class);
        $block = $cms->page('dashboard_experiment');

        return view('dashboard', [
            'iss' => $iss,
            'cms_html' => $cms->safeHtml($block->body ?? '<p>Блок не найден</p>'),
            'trend' => $trend,
            'jw_gallery' => [], // не нужно сервером
            'jw_observation_raw' => [],
            'jw_observation_summary' => [],
            'jw_observation_images' => [],
            'jw_observation_files' => [],
            'metrics' => [
                'iss_speed' => $iss['payload']['velocity'] ?? null,
                'iss_alt'   => $iss['payload']['altitude'] ?? null,
                'neo_total' => 0,
            ],
        ]);
    }

    /**
     * /api/jwst/feed — серверный прокси/нормализатор JWST картинок.
     * QS:
     *  - source: jpg|suffix|program (default jpg)
     *  - suffix: напр. _cal, _thumb, _crf
     *  - program: ID программы (число)
     *  - instrument: NIRCam|MIRI|NIRISS|NIRSpec|FGS
     *  - page, perPage
     */
    public function jwstFeed(Request $r)
    {
        return app(\App\Services\JwstFeedService::class)->feed($r);
    }
}
