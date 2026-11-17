<?php
namespace App\Http\Controllers;
use App\Services\CmsRepository;
class CmsController extends Controller {
    public function page(string $slug, CmsRepository $cms) {
        $row = $cms->page($slug);
        if (!$row) abort(404);
        return response()->view('cms.page', ['title'=>$row->title, 'html'=>$cms->safeHtml($row->body)]);
    }
}
