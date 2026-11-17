<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
use App\Services\AstroService;
class AstroController extends Controller {
    public function events(Request $request, AstroService $service) {
        return response()->json($service->events($request->query()), 200);
    }
}
