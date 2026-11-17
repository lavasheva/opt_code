<?php
namespace App\Http\Controllers;
use App\Services\SpaceClient;
class ProxyController extends Controller {
    public function __construct(private SpaceClient $client) {}
    public function last() { return response()->json($this->client->get('/last')); }
    public function trend() { return response()->json($this->client->get('/iss/trend', request()->query())); }
}
