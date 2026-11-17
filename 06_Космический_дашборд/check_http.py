"""Приёмка локального приложения: python check_http.py после setup.ps1."""
import json
from pathlib import Path
from urllib.request import urlopen

cases = [
    ('http://localhost:8080/dashboard', 'МКС — положение и движение', None),
    ('http://localhost:8080/osdr', 'NASA OSDR', None),
    ('http://localhost:8080/page/welcome', 'Демо контент', None),
    ('http://localhost:8080/page/unsafe', 'Небезопасный пример', None),
    ('http://localhost:8080/api/astro/events?lat=100', None, 'INVALID_COORDINATES'),
    ('http://localhost:8080/api/astro/events', None, None),
    ('http://localhost:8080/api/jwst/feed', None, None),
    ('http://localhost:8081/health', None, None),
    ('http://localhost:8081/last', None, None),
    ('http://localhost:8081/osdr/list', None, None),
    ('http://localhost:8081/space/summary', None, None),
]


def main():
    results = []
    for url, expected_text, expected_code in cases:
        with urlopen(url, timeout=45) as response:
            content = response.read().decode('utf-8')
            assert response.status == 200, url
            if expected_text:
                assert expected_text in content, url
            if '/page/unsafe' in url:
                assert 'XSS training' not in content, 'Injected script survived'
            if 'application/json' in response.headers.get('Content-Type', ''):
                data = json.loads(content)
                if data.get('ok') is False:
                    assert data['error']['code'] and data['error']['trace_id']
                if expected_code:
                    assert data['error']['code'] == expected_code
                preview = data
            else:
                preview = 'HTML checked'
            results.append({'url': url, 'status': response.status, 'body': preview})
            print('PASS', url)
    Path(__file__).with_name('http-results.json').write_text(
        json.dumps(results, ensure_ascii=False, indent=2), encoding='utf-8')
    print(f'PASS: {len(results)} HTTP scenarios')


if __name__ == '__main__':
    main()
