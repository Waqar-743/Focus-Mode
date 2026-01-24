export function SessionTimeline() {
    // Visual blocks for the timeline - matches the design aesthetic
    const visualBlocks = [
        { type: 'break', width: '12%' },
        { type: 'focus', width: '22%' },
        { type: 'break', width: '15%' },
        { type: 'focus', width: '18%' },
        { type: 'break', width: '8%' },
        { type: 'focus', width: '14%' },
        { type: 'break', width: '15%' },
        { type: 'focus', width: '20%' },
    ];

    const timeMarkers = ['Sun', '04:00', '08:00', '12:00', '05:00', '08:00', '12:00'];

    return (
        <div className="timeline">
            <h3 className="timeline__title">Session Timeline</h3>
            <div className="timeline__card">
                <div className="timeline__blocks">
                    {visualBlocks.map((block, idx) => (
                        <div
                            key={idx}
                            className={`timeline__block ${block.type === 'focus'
                                    ? 'timeline__block--focus'
                                    : 'timeline__block--break'
                                }`}
                            style={{ width: block.width }}
                        />
                    ))}
                </div>
                <div className="timeline__markers">
                    {timeMarkers.map((marker, idx) => (
                        <span key={idx} className="timeline__marker">{marker}</span>
                    ))}
                </div>
            </div>
        </div>
    );
}
